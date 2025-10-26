use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use crate::ast::{Expr, LiteralExpr, BinaryExpr, GroupingExpr, UnaryExpr, VariableExpr, CallExpr, Stmt, ExpressionStmt, VarDeclStmt, //Import AST Nodes
BlockStmt, IfStmt, ForStmt, WhileStmt, FunDeclStmt, ReturnStmt}; // <-- ADDED Stmt, ExpressionStmt, VarDeclStmt

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

// Precedences 
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    None,
    Assignment, // = += -= *= /= %= **=
    Or,         // or ||
    And,        // and &&
    Equality,   // == === !=
    Comparison, // > < >= <=
    Term,       // + -
    Factor,     // * / %
    Power,      // **
    Unary,      // ! not -
    Call,       // @ . () []
}
// Helper function to get the precedence of a TokenKind
fn get_precedence(kind: &TokenKind) -> Precedence {
    match kind {
        // ASSIGNMENT (Right-associative, handled specially)
        TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq | TokenKind::StarEq |
        TokenKind::SlashEq | TokenKind::PercentEq | TokenKind::DoubleStarEq => Precedence::Assignment,
        
        // LOGICAL
        TokenKind::Or => Precedence::Or,
        TokenKind::And => Precedence::And,

        // EQUALITY
        TokenKind::EqEq | TokenKind::BangEq | TokenKind::StrictEq => Precedence::Equality,

        // COMPARISON
        TokenKind::Greater | TokenKind::Less | TokenKind::GreaterEq | TokenKind::LessEq => Precedence::Comparison,

        // TERM
        TokenKind::Plus | TokenKind::Minus => Precedence::Term,

        // FACTOR
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,

        // POWER (Right-associative)
        TokenKind::DoubleStar => Precedence::Power,
        
        // CALL / INDEXING
        TokenKind::At | TokenKind::LParen | TokenKind::Dot => Precedence::Call,

        _ => Precedence::None,
    }
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
                // Synchronization: If error, skip to next likely statement start or semicolon
                self.synchronize();
            }
        }
        statements
    }

    // Helper to synchronize parser state after an error
    fn synchronize(&mut self) {
        self.next_token();

        while self.current_token.kind != TokenKind::EOF {
            if self.check_current_kind(&TokenKind::Semicolon) {
                self.next_token();
                return;
            }

            match self.current_token.kind {
                TokenKind::Let | TokenKind::Var | TokenKind::Const | TokenKind::Fun |
                TokenKind::If | TokenKind::While | TokenKind::For |
                TokenKind::EndIf | TokenKind::EndFun | TokenKind::EndFor | TokenKind::EndWhile => return,
                _ => self.next_token(),
            }
        }
    }

    // --- Statement Parsing Function ---
    // This function will determine which type of statement is next.
    fn parse_statement(&mut self) -> Option<Stmt> {
    match self.current_token.kind {
        TokenKind::Let | TokenKind::Var | TokenKind::Const => self.parse_var_declaration(),
        TokenKind::If => self.parse_if_statement(), // NEW
        TokenKind::For => self.parse_for_statement(), // NEW
        TokenKind::While => self.parse_while_statement(), // NEW
        TokenKind::Fun => self.parse_fun_declaration(), // NEW
        TokenKind::Return => self.parse_return_statement(), // NEW
        TokenKind::Break => self.parse_break_statement(), // NEW
        TokenKind::Continue => self.parse_continue_statement(), //NEW
        // Any other statement that is just an expression (like write() or a call)
        _ => self.parse_expression_statement(),
    }
}

    // --- Variable Declaration Parsing Function ---
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

    // --- Expression Statement Parsing Function ---
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

    // --- NEW: Block Parsing ---
    // Reads statements until an 'end...' token (e.g., 'endif', 'endfun') is found.
    fn parse_block(&mut self) -> Option<BlockStmt> {
        let mut statements = Vec::new();
        let termination_keywords = [
            TokenKind::EndIf, TokenKind::EndElif, TokenKind::Else,
            TokenKind::EndFor, TokenKind::EndWhile, TokenKind::EndFun
        ];

        while self.current_token.kind != TokenKind::EOF && !termination_keywords.contains(&self.current_token.kind) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Attempt to synchronize and continue if a single statement failed
                self.synchronize(); 
            }
        }
        Some(BlockStmt { statements })
    }

    // --- NEW: If Statement Parsing ---
    // Handles IF expression THEN Block (ELIF expression THEN Block)* (ELSE Block)? ENDIF
    fn parse_if_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'if'

        // 1. Parse CONDITION
        let condition = self.parse_expression().unwrap_or_else(|| {
            self.error_at_current("Expected condition after 'if'.");
            // Placeholder: Use a boolean literal false to allow parsing to continue
            Expr::Literal(LiteralExpr::Boolean(false)) 
        });

        // 2. Expect 'then' keyword
        if !self.expect_peek(TokenKind::Then) {
            return None; 
        }
        self.next_token(); // Consume 'then'

        // 3. Parse THEN block
        let then_block = self.parse_block().unwrap_or_else(|| {
            self.error_at_current("Expected statement block after 'then'.");
            BlockStmt { statements: vec![] }
        });

        // 4. Check for ELIF/ELSE/ENDIF
        let mut else_branch: Option<Box<Stmt>> = None;
        
        if self.check_current_kind(&TokenKind::Elif) {
            // Recursive call for ELIF: treat elif as the else_branch of the current if
            self.next_token(); // Consume 'elif'
            
            // This is slightly complex: we wrap the ELIF logic back into an IF statement AST node
            if let Some(elif_stmt) = self.parse_if_statement() {
                else_branch = Some(Box::new(elif_stmt));
            } else {
                return None; // Elif statement structure was invalid
            }

        } else if self.check_current_kind(&TokenKind::Else) {
            self.next_token(); // Consume 'else'
            
            let else_block = self.parse_block().unwrap_or_else(|| {
                self.error_at_current("Expected statement block after 'else'.");
                BlockStmt { statements: vec![] }
            });
            else_branch = Some(Box::new(Stmt::If(IfStmt { // Wrapping the else block as a single Stmt
                condition: Expr::Literal(LiteralExpr::Boolean(true)), // Dummy condition for the final ELSE block
                then_branch: Box::new(Stmt::If(IfStmt { // The actual ELSE block statements
                    condition: Expr::Literal(LiteralExpr::Boolean(true)), // Dummy condition
                    then_branch: Box::new(Stmt::Block(else_block)),
                    else_branch: None,
                })),
                else_branch: None,
            })));
        }

        // 5. Expect 'endif' keyword after the whole structure
        if !self.check_current_kind(&TokenKind::EndIf) {
            self.error_at_current("Expected 'endif' to close the if statement.");
            return None;
        }
        self.next_token(); // Consume 'endif'

        Some(Stmt::If(IfStmt {
            condition,
            then_branch: Box::new(Stmt::Block(then_block)),
            else_branch,
        }))
    }

    // --- NEW: For Loop Statement Parsing ---
    // Handles FOR (var_decl | expression) expression expression THEN Block ENDFOR
    fn parse_for_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'for'
        
        // 1. Parse INITIALIZER
        let initializer = if !self.check_current_kind(&TokenKind::Semicolon) {
            self.parse_var_declaration()
                .map(|stmt| Box::new(stmt))
                .or_else(|| self.parse_expression().map(|expr| Box::new(Stmt::Expression(ExpressionStmt { expression: expr }))))
        } else {
            None
        };
        
        // 2. Parse CONDITION (assume it must be present for simplicity)
        let condition = self.parse_expression().unwrap_or_else(|| {
            self.error_at_current("Expected loop condition after initializer.");
            Expr::Literal(LiteralExpr::Boolean(false))
        });
        
        // 3. Parse INCREMENT (optional)
        let increment = self.parse_expression();

        // 4. Expect 'then' keyword
        if !self.expect_peek(TokenKind::Then) {
            return None;
        }
        self.next_token(); // Consume 'then'
        
        // 5. Parse BODY block
        let body_block = self.parse_block().unwrap_or_else(|| {
            self.error_at_current("Expected statement block after 'then' in loop.");
            BlockStmt { statements: vec![] }
        });
        
        // 6. Expect 'endfor' keyword
        if !self.check_current_kind(&TokenKind::EndFor) {
            self.error_at_current("Expected 'endfor' to close the for statement.");
            return None;
        }
        self.next_token(); // Consume 'endfor'

        Some(Stmt::For(ForStmt {
            initializer,
            condition,
            increment,
            body: Box::new(Stmt::Block(body_block)),
        }))
    }

    // --- NEW: While Loop statment Parsing ---
    fn parse_while_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'while'

        // 1. Parse Condition
        let condition = self.parse_expression().unwrap_or_else(|| {
            self.error_at_current("Expected condition after 'while'.");
            Expr::Literal(LiteralExpr::Boolean(false)) // Default to false to avoid crash
        });

        // 2. Expect 'then'
        if !self.expect_peek(TokenKind::Then) { return None; }
        self.next_token(); // Consume 'then'

        // 3. Parse Body Block
        let body_block = self.parse_block()?;

        // 4. Expect 'endwhile'
        if !self.check_current_kind(&TokenKind::EndWhile) {
            self.error_at_current("Expected 'endwhile' to close while loop.");
            return None;
        }
        self.next_token(); // Consume 'endwhile'

        Some(Stmt::While(WhileStmt {
            condition,
            body: Box::new(Stmt::Block(body_block)),
        }))
    }

    fn parse_break_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'break'
        // Optional: check for semicolon here if your grammar requires it
        Some(Stmt::Break)
    }

    fn parse_continue_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'continue'
        // Optional: check for semicolon here if your grammar requires it
        Some(Stmt::Continue)
    }
    

    // --- NEW: Function Parsing Statment ---
    fn parse_fun_declaration(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'fun'

        // 1. Parse Name
        let name = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                self.error_at_current("Expected function name after 'fun'.");
                return None;
            }
        };
        self.next_token(); // Consume function name identifier

        // 2. Parse Parameters (e.g., (p1, p2))
        if !self.expect_peek(TokenKind::LParen) { return None; } // Expect and consume '('
        let parameters = self.parse_identifier_list(TokenKind::RParen); // Helper needed for identifiers
        if !self.check_current_kind(&TokenKind::RParen) { return None; } // After list, current should be ')'
        self.next_token(); // Consume ')'

        // Optional: Type annotation for return value (e.g., : strg) would be checked here

        // 3. Expect 'then'
        if !self.expect_peek(TokenKind::Then) { return None; }
        self.next_token(); // Consume 'then'

        // 4. Parse Body Block
        let body_block = self.parse_block()?;
        
        // 5. Expect 'endfun'
        if !self.check_current_kind(&TokenKind::EndFun) {
            self.error_at_current("Expected 'endfun' to close function definition.");
            return None;
        }
        self.next_token(); // Consume 'endfun'

        Some(Stmt::FunctionDecl(FunDeclStmt {
            name,
            parameters,
            body: Box::new(Stmt::Block(body_block)),
        }))
    }
    
    // Helper needed to parse comma-separated identifiers (parameters)
    fn parse_identifier_list(&mut self, delimiter: TokenKind) -> Vec<String> {
        let mut identifiers = Vec::new();
        
        // If next token is the closing delimiter, the list is empty (e.g., '()')
        if self.check_current_kind(&delimiter) {
            return identifiers;
        }

        loop {
            // 1. Expect Identifier
            let name = match &self.current_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    self.error_at_current("Expected identifier in parameter list.");
                    break; 
                }
            };
            identifiers.push(name);
            self.next_token(); // Consume identifier

            // 2. Check for comma or closing delimiter
            if self.check_current_kind(&TokenKind::Comma) {
                self.next_token(); // Consume ','
            } else if self.check_current_kind(&delimiter) {
                break; // Found closing delimiter
            } else {
                self.error_at_current(&format!("Expected ',' or closing delimiter {:?} in list.", delimiter));
                break; 
            }
        }
        identifiers
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // Consume 'return'

        // Check for expression (optional return value)
        let value = if !self.check_current_kind(&TokenKind::EndFun) && 
                    !self.check_current_kind(&TokenKind::Semicolon) // Assuming return can be followed by ;
        {
            self.parse_expression()
        } else {
            None
        };

        // If your grammar requires a semicolon after return, check it here:
        // if !self.expect_peek(TokenKind::Semicolon) { return None; }

        Some(Stmt::Return(ReturnStmt { value }))
    }


    fn parse_expression_with_precedence(&mut self, precedence: Precedence) -> Option<Expr> {
        // 1. Parse the LHS (Prefix/Primary Expression)
        let mut left_expr = self.parse_prefix_expression()?;

        // 2. Loop while the next operator has higher precedence than the current level
        while precedence < get_precedence(&self.current_token.kind) {
            let operator_kind = self.current_token.kind.clone();
            let operator_precedence = get_precedence(&operator_kind);
            
            // If the operator is right-associative (like Power or Assignment),
            // we use a precedence one level lower for the RHS to force right-to-left grouping.
            let next_precedence = if operator_kind == TokenKind::DoubleStar || operator_kind == TokenKind::Eq {
                operator_precedence
            } else {
                // For left-associative operators, we use the same precedence for the RHS
                // to allow sequential grouping (A + B + C = (A + B) + C).
                operator_precedence
            };


            // Check if the operator is a binary infix operator
            if operator_precedence == Precedence::None {
                // If the current token is not an operator we handle here (e.g., a semicolon or keyword), break the loop.
                break;
            }

            // Consume the infix operator (e.g., +, -, ==)
            self.next_token(); 

            // 3. Parse the RHS (Recursive call)
            let right_expr = self.parse_expression_with_precedence(next_precedence)?;

            // 4. Construct the new binary expression
            left_expr = Expr::Binary(BinaryExpr {
                left: Box::new(left_expr),
                operator: operator_kind,
                right: Box::new(right_expr),
            });
        }

        Some(left_expr)
    }

    // Public entry point to parse_expression (used by parse_statement, etc.)
    pub fn parse_expression(&mut self) -> Option<Expr> {
        // Start with the lowest precedence to parse the whole expression
        self.parse_expression_with_precedence(Precedence::Assignment)
    }


    // Parses the lowest precedence expressions (literals, identifiers, grouped expressions)
    fn parse_prefix_expression(&mut self) -> Option<Expr> {
        match &self.current_token.kind {
            // UNARY OPERATORS
            TokenKind::Not | TokenKind::Minus | TokenKind::Not => {
                let operator = self.current_token.kind.clone();
                self.next_token(); // Consume the operator
                
                // Recursively parse the expression on the right with Call precedence (highest)
                let right_expr = self.parse_expression_with_precedence(Precedence::Unary)?; 
                
                Some(Expr::Unary(UnaryExpr {
                    operator,
                    right: Box::new(right_expr),
                }))
            },

            // LITERALS and GROUPING (These are our base cases for expressions)
            // ... (TokenKind::Integer(val) ... TokenKind::Null, TokenKind::LParen logic remain here, just copy/paste from the old parse_primary_expression)
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
                    return None;
                }
                if let Some(inner_expr) = expr {
                    Some(Expr::Grouping(GroupingExpr { expression: Box::new(inner_expr) }))
                } else {
                    None
                }
            },
            TokenKind::LBracket => { // Array/List literal: [1, 2, 3]
                self.next_token(); // Consume '['
                let elements = self.parse_expression_list(TokenKind::RBracket); // Needs a new helper
                self.expect_peek(TokenKind::RBracket); // Consume ']'
                Some(Expr::Literal(LiteralExpr::Array(elements))) // Assuming LiteralExpr::Array
            },
            TokenKind::LBrace => { // Set/Dict literal: {1, 2} or {"k": "v"}
                self.next_token(); // Consume '{'
                // Placeholder for complex set/dict parsing. For now, we'll parse a simple list.
                let elements = self.parse_expression_list(TokenKind::RBrace);
                self.expect_peek(TokenKind::RBrace); // Consume '}'
                // You will need a new AST type here (e.g., LiteralExpr::Set or LiteralExpr::Dict)
                // For now, let's treat it as a placeholder Set/Dict type:
                Some(Expr::Literal(LiteralExpr::Set(elements))) // Assuming LiteralExpr::Set
            },
            TokenKind::At => {
                self.next_token(); // Consume '@'
                
                // Callee must be an Identifier (for now)
                let callee = self.parse_prefix_expression().map(Box::new)?;

                // Must be followed by '('
                if !self.check_current_kind(&TokenKind::LParen) {
                    self.error_at_current("Expected '(' after function name.");
                    return None;
                }
                self.next_token(); // Consume '('
                
                let arguments = self.parse_expression_list(TokenKind::RParen);
                self.expect_peek(TokenKind::RParen); // Consume ')'

                Some(Expr::Call(CallExpr { callee, arguments }))
            }
            _ => {
                self.error_at_current(&format!("Unexpected token for expression start: {:?}", self.current_token.kind));
                None
            }
        }
    }

    // --- Helper for lists of expressions (arrays, function arguments, etc.) ---
    fn parse_expression_list(&mut self, delimiter: TokenKind) -> Vec<Expr> {
        let mut list = Vec::new();
        // Check if the list is empty right away
        if self.check_current_kind(&delimiter) {
            return list;
        }

        loop {
            if let Some(expr) = self.parse_expression() {
                list.push(expr);
            } else {
                // Expression failed, attempt to recover by breaking or synchronizing
                self.synchronize(); 
                break;
            }

            // Check for continuation (Comma)
            if self.check_current_kind(&TokenKind::Comma) {
                self.next_token(); // Consume ','
            } else {
                break; // No comma means the list ends here
            }
        }
        list
    }
}