use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use crate::ast::{Expr, LiteralExpr, BinaryExpr, GroupingExpr, UnaryExpr, VariableExpr, CallExpr}; // Import AST nodes

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token(); // Peek one token ahead
        Parser {
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    // Helper method to advance the tokens
    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    // Helper for error reporting
    fn error_at_current(&mut self, msg: &str) {
        let err_msg = format!(
            "Parse Error at line {} column {}: {}",
            self.current_token.line, self.current_token.column, msg
        );
        self.errors.push(err_msg);
    }

    // Helper for error reporting at peek token
    fn error_at_peek(&mut self, msg: &str) {
        let err_msg = format!(
            "Parse Error at line {} column {}: {}",
            self.peek_token.line, self.peek_token.column, msg
        );
        self.errors.push(err_msg);
    }

    // Helper to check if current token is of a certain kind
    fn check_current_kind(&self, kind: &TokenKind) -> bool {
        self.current_token.kind == *kind
    }

    // Helper to check if peek token is of a certain kind
    fn check_peek_kind(&self, kind: &TokenKind) -> bool {
        self.peek_token.kind == *kind
    }

    // Helper to expect and consume a token of a certain kind
    fn expect_peek(&mut self, expected_kind: TokenKind) -> bool {
        if self.check_peek_kind(&expected_kind) {
            self.next_token();
            true
        } else {
            self.error_at_peek(&format!("Expected next token to be {:?}, got {:?}", expected_kind, self.peek_token.kind));
            false
        }
    }

    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }

    // --- Parsing Entry Point ---
    // This will be expanded significantly. For now, let's just parse a single expression.
    pub fn parse_expression(&mut self) -> Option<Expr> {
        // This will be where our expression parsing logic starts.
        // For now, let's just implement parsing of literal values.
        self.parse_primary_expression()
    }

    // Parses the lowest precedence expressions (literals, identifiers, grouped expressions)
    fn parse_primary_expression(&mut self) -> Option<Expr> {
        match &self.current_token.kind {
            TokenKind::Integer(val) => {
                let expr = Expr::Literal(LiteralExpr::Integer(*val));
                self.next_token();
                Some(expr)
            },
            TokenKind::Float(val) => {
                let expr = Expr::Literal(LiteralExpr::Float(*val));
                self.next_token();
                Some(expr)
            },
            TokenKind::String(val) => {
                let expr = Expr::Literal(LiteralExpr::String(val.clone()));
                self.next_token();
                Some(expr)
            },
            TokenKind::True => {
                let expr = Expr::Literal(LiteralExpr::Boolean(true));
                self.next_token();
                Some(expr)
            },
            TokenKind::False => {
                let expr = Expr::Literal(LiteralExpr::Boolean(false));
                self.next_token();
                Some(expr)
            },
            TokenKind::Null => {
                let expr = Expr::Literal(LiteralExpr::Null);
                self.next_token();
                Some(expr)
            },
            TokenKind::Identifier(name) => {
                let expr = Expr::Variable(VariableExpr { name: name.clone() });
                self.next_token();
                Some(expr)
            },
            TokenKind::LParen => {
                self.next_token(); // Consume '('
                let expr = self.parse_expression(); // Recursively parse the inner expression
                if !self.expect_peek(TokenKind::RParen) { // Expect and consume ')'
                    return None; // Error, missing ')'
                }
                if let Some(inner_expr) = expr {
                    Some(Expr::Grouping(GroupingExpr { expression: Box::new(inner_expr) }))
                } else {
                    None // Error in inner expression
                }
            },
            _ => {
                self.error_at_current(&format!("Unexpected token for primary expression: {:?}", self.current_token.kind));
                None // Indicates parsing failure for this rule
            }
        }
    }
}