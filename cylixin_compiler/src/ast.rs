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


// We'll also need statements later, but let's keep it simple for now.
// pub enum Stmt {
//     Expression(Expr),
//     VarDeclaration(VarDeclStmt),
//     If(IfStmt),
//     // ... other statements
// }