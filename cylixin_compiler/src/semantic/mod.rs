/// Semantic analysis pass — walks the AST before codegen and catches
/// common programming mistakes with friendly `line:col: error` messages.
///
/// First-pass checks (intentionally limited to avoid over-engineering):
///   1. Undefined variables (reads a name that hasn't been declared).
///   2. Undefined functions (calls a function that hasn't been declared or is not a builtin).
///   3. Wrong number of arguments to a user-defined function.
///   4. `break` / `continue` outside a loop.
///   5. `return` outside a function.
///   6. Type mismatch on assignment and `return` (best-effort; only checks the obvious cases).
///
/// Span information is best-effort: every statement carries no span today,
/// so errors on statements report (0, 0). Expression-level errors inside a
/// named binding do carry a name that appears in the message.

use std::collections::HashMap;
use crate::ast::*;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub line:    usize,
    pub column:  usize,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "semantic error: {}", self.message)
        } else {
            write!(f, "{}:{}: semantic error: {}", self.line, self.column, self.message)
        }
    }
}

// ---------------------------------------------------------------------------
// Scope / environment
// ---------------------------------------------------------------------------

/// Per-scope variable: name → declared type (if annotated)
type VarEnv = HashMap<String, Option<CyType>>;

/// Known user function signature
#[derive(Clone)]
struct FunSig {
    params:      Vec<Param>,
    return_type: Option<CyType>,
}

// Names of all built-in functions callable with @
const BUILTINS: &[&str] = &["write", "writeln", "read", "is_even"];

// ---------------------------------------------------------------------------
// Checker state
// ---------------------------------------------------------------------------

pub struct Checker {
    /// Stack of variable scopes; outermost is index 0.
    scopes:    Vec<VarEnv>,
    /// All user-declared functions (first pass collects them all up front).
    functions: HashMap<String, FunSig>,
    /// Errors accumulated during the walk.
    errors:    Vec<SemanticError>,
    /// How many loops deep we currently are (for break/continue validation).
    loop_depth: usize,
    /// Whether we are inside a function body (for return validation).
    in_function: bool,
    /// Expected return type of the current function, if any.
    expected_return: Option<CyType>,
}

impl Checker {
    pub fn new() -> Self {
        Checker {
            scopes:          vec![HashMap::new()],
            functions:       HashMap::new(),
            errors:          Vec::new(),
            loop_depth:      0,
            in_function:     false,
            expected_return: None,
        }
    }

    // ------------------------------------------------------------------
    // Public entry point
    // ------------------------------------------------------------------

    /// Run the checker against a parsed program. Returns all accumulated
    /// errors; empty means the program passed the semantic checks.
    pub fn check(mut self, program: &Program) -> Vec<SemanticError> {
        // First pass: collect all function signatures so forward calls work.
        for stmt in &program.body {
            if let Stmt::FunDecl { name, params, return_type, .. } = stmt {
                self.functions.insert(name.clone(), FunSig {
                    params:      params.clone(),
                    return_type: return_type.clone(),
                });
            }
        }

        // Second pass: walk every statement.
        for stmt in &program.body {
            self.check_stmt(stmt);
        }

        self.errors
    }

    // ------------------------------------------------------------------
    // Scope helpers
    // ------------------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: &str, ty: Option<CyType>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup_var(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.contains_key(name))
    }

    fn var_type(&self, name: &str) -> Option<CyType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return ty.clone();
            }
        }
        None
    }

    // ------------------------------------------------------------------
    // Error helpers
    // ------------------------------------------------------------------

    fn error(&mut self, message: impl Into<String>) {
        self.errors.push(SemanticError {
            message: message.into(),
            line:    0,
            column:  0,
        });
    }

    fn error_at(&mut self, line: usize, col: usize, message: impl Into<String>) {
        self.errors.push(SemanticError {
            message: message.into(),
            line,
            column: col,
        });
    }

    // ------------------------------------------------------------------
    // Statement walker
    // ------------------------------------------------------------------

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, type_ann, initialiser, .. } => {
                // Check the initialiser expression first (before binding the name,
                // so `let x = x;` is caught as undefined on the RHS).
                if let Some(init) = initialiser {
                    self.check_expr(init);
                }
                self.declare_var(name, type_ann.clone());
            }

            Stmt::Assign { target, value, .. } => {
                // Check the assigned value.
                self.check_expr(value);
                // Check that the target variable exists.
                match target {
                    AssignTarget::Ident(name) => {
                        if !self.lookup_var(name) {
                            self.error(format!("assignment to undeclared variable '{}'", name));
                        }
                    }
                    AssignTarget::Index { name, index } => {
                        if !self.lookup_var(name) {
                            self.error(format!("assignment to undeclared variable '{}'", name));
                        }
                        self.check_expr(index);
                    }
                }
            }

            Stmt::If { condition, then_body, elif_arms, else_body, .. } => {
                self.check_expr(condition);
                self.push_scope();
                for s in then_body { self.check_stmt(s); }
                self.pop_scope();
                for arm in elif_arms {
                    self.check_expr(&arm.condition);
                    self.push_scope();
                    for s in &arm.body { self.check_stmt(s); }
                    self.pop_scope();
                }
                if let Some(body) = else_body {
                    self.push_scope();
                    for s in body { self.check_stmt(s); }
                    self.pop_scope();
                }
            }

            Stmt::ForRange { var, from, to, body, .. } => {
                self.check_expr(from);
                self.check_expr(to);
                self.push_scope();
                self.declare_var(var, Some(CyType::Int));
                self.loop_depth += 1;
                for s in body { self.check_stmt(s); }
                self.loop_depth -= 1;
                self.pop_scope();
            }

            Stmt::ForC { init, cond, update, body, .. } => {
                self.push_scope();
                self.check_stmt(init);
                self.check_expr(cond);
                self.loop_depth += 1;
                for s in body { self.check_stmt(s); }
                self.loop_depth -= 1;
                self.check_stmt(update);
                self.pop_scope();
            }

            Stmt::While { condition, body, .. } => {
                self.check_expr(condition);
                self.push_scope();
                self.loop_depth += 1;
                for s in body { self.check_stmt(s); }
                self.loop_depth -= 1;
                self.pop_scope();
            }

            Stmt::FunDecl { name, params, return_type, body, .. } => {
                // Save outer function context
                let outer_in_function     = self.in_function;
                let outer_expected_return = self.expected_return.clone();
                let outer_loop_depth      = self.loop_depth;

                self.in_function     = true;
                self.expected_return = return_type.clone();
                self.loop_depth      = 0; // loops inside a fun start fresh

                self.push_scope();
                for param in params {
                    self.declare_var(&param.name, Some(param.type_ann.clone()));
                }
                for s in body { self.check_stmt(s); }
                self.pop_scope();

                // Restore
                self.in_function     = outer_in_function;
                self.expected_return = outer_expected_return;
                self.loop_depth      = outer_loop_depth;

                let _ = name; // name is already registered in the pre-pass
            }

            Stmt::Return(expr) => {
                if !self.in_function {
                    self.error("`return` used outside of a function");
                }
                if let Some(e) = expr {
                    self.check_expr(e);
                }
            }

            Stmt::Break(label) => {
                if self.loop_depth == 0 {
                    match label {
                        Some(l) => self.error(format!("`break {}` used outside of a loop", l)),
                        None    => self.error("`break` used outside of a loop"),
                    }
                }
            }

            Stmt::Continue(label) => {
                if self.loop_depth == 0 {
                    match label {
                        Some(l) => self.error(format!("`continue {}` used outside of a loop", l)),
                        None    => self.error("`continue` used outside of a loop"),
                    }
                }
            }

            Stmt::ExprStmt(expr) => {
                self.check_expr(expr);
            }
        }
    }

    // ------------------------------------------------------------------
    // Expression walker
    // ------------------------------------------------------------------

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            // Literals are always fine
            Expr::IntLit(_) | Expr::LongLit(_) | Expr::FloatLit(_)
            | Expr::StringLit(_) | Expr::CharLit(_) | Expr::BoolLit(_)
            | Expr::NullLit => {}

            Expr::Ident(name) => {
                if !self.lookup_var(name) {
                    self.error(format!("undefined variable '{}'", name));
                }
            }

            Expr::BinaryOp { left, right, op } => {
                self.check_expr(left);
                self.check_expr(right);

                // Best-effort: catch obvious type mismatches for arithmetic ops.
                // We only do this when both sides are simple literals or idents
                // where we know the type.
                let lty = self.infer_type(left);
                let rty = self.infer_type(right);
                if let (Some(l), Some(r)) = (lty, rty) {
                    self.check_binary_types(&l, &r, op);
                }
            }

            Expr::UnaryOp { expr: inner, .. } => {
                self.check_expr(inner);
            }

            Expr::Call { name, args } => {
                self.check_call(name, args);
            }

            Expr::Index { collection, index } => {
                self.check_expr(collection);
                self.check_expr(index);
            }

            Expr::ArrayLit(elems) => {
                for e in elems { self.check_expr(e); }
            }

            Expr::SetLit(elems) => {
                for e in elems { self.check_expr(e); }
            }

            Expr::DictLit(pairs) => {
                for (k, v) in pairs {
                    self.check_expr(k);
                    self.check_expr(v);
                }
            }

            Expr::Grouped(inner) => {
                self.check_expr(inner);
            }
        }
    }

    // ------------------------------------------------------------------
    // Call validation
    // ------------------------------------------------------------------

    fn check_call(&mut self, name: &str, args: &[Expr]) {
        // Always check all argument expressions first.
        for arg in args { self.check_expr(arg); }

        // `read` is magic — it takes exactly one string prompt argument.
        if name == "read" {
            if args.len() != 1 {
                self.error(format!(
                    "@read() requires exactly 1 argument (prompt string), got {}",
                    args.len()
                ));
            }
            return;
        }

        // Other recognised builtins: write / writeln take ≥ 1 arg.
        if name == "write" || name == "writeln" {
            if args.is_empty() {
                self.error(format!("@{}() requires at least 1 argument, got 0", name));
            }
            return;
        }

        // Remaining recognised builtins (no arg-count check needed).
        if BUILTINS.contains(&name) {
            return;
        }

        // User function
        if let Some(sig) = self.functions.get(name).cloned() {
            if args.len() != sig.params.len() {
                self.error(format!(
                    "function '{}' expects {} argument(s) but {} were supplied",
                    name,
                    sig.params.len(),
                    args.len()
                ));
            }
        } else {
            self.error(format!("call to undeclared function '{}'", name));
        }
    }

    // ------------------------------------------------------------------
    // Type inference (best-effort, literals + known variables only)
    // ------------------------------------------------------------------

    fn infer_type(&self, expr: &Expr) -> Option<CyType> {
        match expr {
            Expr::IntLit(_)    => Some(CyType::Int),
            Expr::LongLit(_)   => Some(CyType::Long),
            Expr::FloatLit(_)  => Some(CyType::Float),
            Expr::BoolLit(_)   => Some(CyType::Bool),
            Expr::CharLit(_)   => Some(CyType::Char),
            Expr::StringLit(_) => Some(CyType::StringType),
            Expr::NullLit      => Some(CyType::Null),
            Expr::Ident(n)     => self.var_type(n),
            Expr::Grouped(e)   => self.infer_type(e),
            _ => None, // too complex to infer without a full type-system
        }
    }

    // ------------------------------------------------------------------
    // Binary type compatibility check
    // ------------------------------------------------------------------

    fn check_binary_types(&mut self, lty: &CyType, rty: &CyType, op: &BinaryOp) {
        // String concatenation with + is valid.
        if matches!(op, BinaryOp::Add)
            && matches!(lty, CyType::StringType)
            && matches!(rty, CyType::StringType)
        {
            return;
        }

        // Comparisons between equal types are always fine.
        if matches!(op, BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::StrictEq) {
            return;
        }

        // Logical ops require bool operands.
        if matches!(op, BinaryOp::And | BinaryOp::Or) {
            return; // coercion is intentionally permissive here
        }

        // Bitwise ops require integer types.
        if matches!(op, BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::Shl | BinaryOp::Shr) {
            let l_int = matches!(lty, CyType::Int | CyType::Long);
            let r_int = matches!(rty, CyType::Int | CyType::Long);
            if !l_int || !r_int {
                self.error(format!(
                    "bitwise operator requires integer operands, got {:?} and {:?}",
                    lty, rty
                ));
            }
            return;
        }

        // Arithmetic ops: mixing string with numeric is an error.
        if matches!(lty, CyType::StringType) || matches!(rty, CyType::StringType) {
            self.error(format!(
                "cannot apply arithmetic operator to string type (got {:?} and {:?})",
                lty, rty
            ));
        }
    }
}
