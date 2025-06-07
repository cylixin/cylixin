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


// New: enum for statements
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(ExpressionStmt),  // An expression treated as a statement (e.g., `1 + 2;`)
    VarDeclaration(VarDeclStmt), // Variable declaration (e.g., `let x = 5;`)
    // We'll add more statement types later (If, While, Function, etc.)
}

// New: Struct for an expression statement
#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionStmt {
    pub expression: Expr, // The expression to evaluate
}

// New: Struct for variable declaration statement
#[derive(Debug, PartialEq, Clone)]
pub struct VarDeclStmt {
    pub name: String,  // Name of the variable
    pub initializer: Option<Expr>,  // Optional initail value
}