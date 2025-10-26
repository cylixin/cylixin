use crate::token::TokenKind;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Binary(BinaryExpr),
    Grouping(GroupingExpr),
    Unary(UnaryExpr),
    Variable(VariableExpr), // For identifiers
    Call(CallExpr), // For function calls
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralExpr {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Expr>),
    Set(Vec<Expr>),
    Dictionary(Vec<(Expr, Expr)>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: TokenKind, // The operator token (e.g., Plus, Minus)
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpr {
    pub operator: TokenKind, // The operator token (e.g., Minus, Not)
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableExpr {
    pub name: String, // The identifier's name
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>, // The expression being called (e.g., a variable for a function name)
    pub arguments: Vec<Expr>,
}


// enum for statements
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),  // An expression treated as a statement (e.g., `1 + 2;`)
    VarDeclaration(VarDeclStmt), // Variable declaration (e.g., `let x = 5;`)
    Block(BlockStmt),            // For making code blocks like loops, functions, if/else etc
    If(IfStmt),                  // If, else, and elif statements
    For(ForStmt),                // Handles for loops
    While(WhileStmt),            // Handles while loops
    FunctionDecl(FunDeclStmt),   // For declaring functions
    Return(ReturnStmt),          // return expression
    Break,                       // break
    Continue,                    // continue
    // We'll add more statement types later (If, While, Function, etc.)
}

// Struct for an expression statement
#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionStmt {
    pub expression: Expr, // The expression to evaluate
}

// Struct for variable declaration statement
#[derive(Debug, PartialEq, Clone)]
pub struct VarDeclStmt {
    pub name: String,  // Name of the variable
    pub initializer: Option<Expr>,  // Optional initail value
}

// New: Struct for code blocks {...}
#[derive(Debug, PartialEq, Clone)]
pub struct BlockStmt {
    pub statements: Vec<Stmt>, 
}

// New: Struct for if, elif, and else blocks
#[derive(Debug, PartialEq, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,          // The BlockStmt inside the 'then'
    pub else_branch: Option<Box<Stmt>>,  // The optional 'elif' (as a nested IfStmt) or 'else' (as a nested BlockStmt)
}

// New: Handles for [initializer] condition [increment] then Block endfor
#[derive(Debug, PartialEq, Clone)]
pub struct ForStmt {
    pub initializer: Option<Box<Stmt>>,
    pub condition: Expr,
    pub increment: Option<Expr>,
    pub body: Box<Stmt>,
}

// New: Handles while (condition) then Block endwhile
#[derive(Debug, PartialEq, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

// New: Handles fun name(args) then Block endfun
#[derive(Debug, PartialEq, Clone)]
pub struct FunDeclStmt {
    pub name: String,
    pub parameters: Vec<String>,   // Parameter names
    pub body: Box<Stmt>,           // The Blockstmt inside the function
}

// New: Handles Return Statement
#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,       // The expression being returned (optional)
}