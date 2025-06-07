use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use crate::ast::{Expr, LiteralExpr, BinaryExpr, GroupingExpr, UnaryExpr, VariableExpr, CallExpr,   // Import AST Nodes
                 Stmt, ExpressionStmt, VarDeclStmt}; // <-- ADDED Stmt, ExpressionStmt, VarDeclStmt

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

    // --- New: Main Program Parsing Entry Point ---
    // This function will parse a sequence of statements until EOF.
    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.current_token.kind != TokenKind::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // If a statement couldn't be parsed, advance past the current token
                // to try and recover and parse the next statement.
                self.next_token();
            }
        }
        statements
    }

    // --- New: Statement Parsing Function ---
    // This function will determine which type of statement is next.
    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token.kind {
            TokenKind::Let | TokenKind::Var | TokenKind::Const => self.parse_var_declaration(),
            _ => self.parse_expression_statement(), // Fallback: try to parse an expression statement
        }
    }

    // --- New: Variable Declaration Parsing Function ---
    // Handles `let identifier = expression;`
    fn parse_var_declaration(&mut self) -> Option<Stmt> {
        let keyword_token = self.current_token.clone(); // `let`, `var`, or `const`
        self.next_token(); // Consume `let`/`var`/`const`

        // Expect an identifier for the variable name
        let name_token = if let TokenKind::Identifier(name) = &self.current_token.kind {
            name.clone() // Get the name string
        } else {
            self.error_at_current("Expected variable name after 'let', 'var', or 'const'.");
            return None;
        };
        self.next_token(); // Consume the identifier

        let mut initializer: Option<Expr> = None;
        // Check for an initializer (e.g., `= expression`)
        if self.check_current_kind(&TokenKind::Eq) { // Check if current token is `=`
            self.next_token(); // Consume `=`
            if let Some(expr) = self.parse_expression() { // Parse the initializer expression
                initializer = Some(expr);
            } else {
                self.error_at_current("Expected expression after '=' in variable declaration.");
                return None;
            }
        }

        // In Cylixin, statements don't seem to end with a semicolon, but if they did,
        // we'd expect it here. For now, we'll assume it's optional or handled implicitly.
        // If your grammar requires a semicolon:
        // if !self.expect_peek(TokenKind::Semicolon) {
        //     return None;
        // }


        Some(Stmt::VarDeclaration(VarDeclStmt {
            name: name_token,
            initializer,
        }))
    }

    // --- New: Expression Statement Parsing Function ---
    // Handles expressions that stand alone as statements (e.g., `1 + 2;` or `myFunction();`)
    fn parse_expression_statement(&mut self) -> Option<Stmt> {
        if let Some(expr) = self.parse_expression() {
            // If your grammar requires a semicolon after expression statements:
            // if !self.expect_peek(TokenKind::Semicolon) {
            //     return None;
            // }
            Some(Stmt::Expression(ExpressionStmt { expression: expr }))
        } else {
            // If parse_expression failed, we can't form an ExpressionStmt
            None
        }
    }


    // --- Existing: Primary Expression Parsing Function ---
    pub fn parse_expression(&mut self) -> Option<Expr> {
        // This will eventually call a more complex expression parsing function
        // that handles operator precedence. For now, it only parses primary expressions.
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