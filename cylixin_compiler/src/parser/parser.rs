use crate::lexer::token::{Token, TokenKind};
use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: String, line: usize, column: usize },
    UnexpectedEof   { context: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, line, column } =>
                write!(f, "Expected {} but found {} at {}:{}", expected, found, line, column),
            ParseError::UnexpectedEof { context } =>
                write!(f, "Unexpected end of file while parsing {}", context),
        }
    }
}

pub struct Parser {
    tokens:  Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut body = Vec::new();
        while !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        Ok(Program::new(body))
    }

    // dispatch based on the current token
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::Let | TokenKind::Var | TokenKind::Const => self.parse_var_decl(),
            TokenKind::If     => self.parse_if(),
            TokenKind::For    => self.parse_for(),
            TokenKind::While  => self.parse_while(),
            TokenKind::Fun    => self.parse_fun_decl(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break    => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::Identifier(_) => self.parse_assign_or_expr_stmt(),
            TokenKind::At => {
                let expr = self.parse_expression()?;
                self.expect_semicolon()?;
                Ok(Stmt::ExprStmt(expr))
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect_semicolon()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        let kind = match self.advance().kind.clone() {
            TokenKind::Let   => VarKind::Let,
            TokenKind::Var   => VarKind::Var,
            TokenKind::Const => VarKind::Const,
            _ => unreachable!(),
        };

        let name = self.expect_identifier("variable name")?;

        let type_ann = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let initialiser = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_semicolon()?;
        Ok(Stmt::VarDecl { kind, name, type_ann, initialiser })
    }

    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect_identifier("variable or expression")?;

        match self.peek_kind() {
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket, "]")?;
                let op    = self.parse_assign_op()?;
                let value = self.parse_expression()?;
                self.expect_semicolon()?;
                Ok(Stmt::Assign { target: AssignTarget::Index { name, index }, op, value })
            }
            TokenKind::Eq     | TokenKind::PlusEq  | TokenKind::MinusEq |
            TokenKind::StarEq | TokenKind::SlashEq | TokenKind::PercentEq |
            TokenKind::StarStarEq => {
                let op    = self.parse_assign_op()?;
                let value = self.parse_expression()?;
                self.expect_semicolon()?;
                Ok(Stmt::Assign { target: AssignTarget::Ident(name), op, value })
            }
            _ => {
                // not an assignment - treat as expression statement
                let expr = self.parse_expression_from(Expr::Ident(name))?;
                self.expect_semicolon()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_assign_op(&mut self) -> Result<AssignOp, ParseError> {
        let op = match self.peek_kind() {
            TokenKind::Eq         => AssignOp::Assign,
            TokenKind::PlusEq     => AssignOp::AddAssign,
            TokenKind::MinusEq    => AssignOp::SubAssign,
            TokenKind::StarEq     => AssignOp::MulAssign,
            TokenKind::SlashEq    => AssignOp::DivAssign,
            TokenKind::PercentEq  => AssignOp::ModAssign,
            TokenKind::StarStarEq => AssignOp::PowAssign,
            other => {
                let tok = self.peek();
                return Err(ParseError::UnexpectedToken {
                    expected: "assignment operator".into(),
                    found:    format!("{:?}", other),
                    line:     tok.span.line,
                    column:   tok.span.column,
                });
            }
        };
        self.advance();
        Ok(op)
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::If, "if")?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Then, "then")?;

        let terminators = &[TokenKind::Elif, TokenKind::Else, TokenKind::EndIf];
        let then_body   = self.parse_block(terminators)?;

        let mut elif_arms = Vec::new();
        while self.check(&TokenKind::Elif) {
            self.advance();
            let cond = self.parse_expression()?;
            self.expect(TokenKind::Then, "then")?;
            let body = self.parse_block(terminators)?;
            elif_arms.push(ElifArm { condition: cond, body });
        }

        let else_body = if self.check(&TokenKind::Else) {
            self.advance();
            Some(self.parse_block(&[TokenKind::EndIf])?)
        } else {
            None
        };

        self.expect(TokenKind::EndIf, "endif")?;
        let end_when = self.parse_end_when()?;

        Ok(Stmt::If { condition, then_body, elif_arms, else_body, end_when })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::For, "for")?;
        let label = self.try_parse_label();
        let var   = self.expect_identifier("loop variable")?;

        if self.check(&TokenKind::From) {
            self.advance();
            let from = self.parse_expression()?;
            self.expect(TokenKind::To, "to")?;
            let to = self.parse_expression()?;
            self.expect(TokenKind::Then, "then")?;

            let body     = self.parse_block(&[TokenKind::EndFor])?;
            self.expect(TokenKind::EndFor, "endfor")?;
            let end_when = self.parse_end_when()?;

            Ok(Stmt::ForRange { var, from, to, body, label, end_when })
        } else {
            // C-style:  for i = 0; i < n; i += 1 then
            let init_op  = self.parse_assign_op()?;
            let init_val = self.parse_expression()?;
            self.expect(TokenKind::Semicolon, ";")?;

            let init = Box::new(Stmt::Assign {
                target: AssignTarget::Ident(var.clone()),
                op: init_op, value: init_val,
            });

            let cond = self.parse_expression()?;
            self.expect(TokenKind::Semicolon, ";")?;

            let update_name = self.expect_identifier("update variable")?;
            let update_op   = self.parse_assign_op()?;
            let update_val  = self.parse_expression()?;
            let update = Box::new(Stmt::Assign {
                target: AssignTarget::Ident(update_name),
                op: update_op, value: update_val,
            });

            self.expect(TokenKind::Then, "then")?;
            let body     = self.parse_block(&[TokenKind::EndFor])?;
            self.expect(TokenKind::EndFor, "endfor")?;
            let end_when = self.parse_end_when()?;

            Ok(Stmt::ForC { init, cond, update, body, label, end_when })
        }
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::While, "while")?;
        let label     = self.try_parse_label();
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Then, "then")?;

        let body     = self.parse_block(&[TokenKind::EndWhile])?;
        self.expect(TokenKind::EndWhile, "endwhile")?;
        let end_when = self.parse_end_when()?;

        Ok(Stmt::While { condition, body, label, end_when })
    }

    fn parse_fun_decl(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Fun, "fun")?;
        let name = self.expect_identifier("function name")?;

        self.expect(TokenKind::LParen, "(")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, ")")?;

        let return_type = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Then, "then")?;
        let body = self.parse_block(&[TokenKind::EndFun])?;
        self.expect(TokenKind::EndFun, "endfun")?;
        self.parse_end_when()?;

        Ok(Stmt::FunDecl { name, params, return_type, body })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) { return Ok(params); }
        loop {
            let name     = self.expect_identifier("parameter name")?;
            self.expect(TokenKind::Colon, ":")?;
            let type_ann = self.parse_type()?;
            params.push(Param { name, type_ann });
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        Ok(params)
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Return, "return")?;
        let value = if self.check(&TokenKind::Semicolon) { None }
                    else { Some(self.parse_expression()?) };
        self.expect_semicolon()?;
        Ok(Stmt::Return(value))
    }

    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Break, "break")?;
        let label = if let TokenKind::Identifier(_) = self.peek_kind() {
            Some(self.expect_identifier("break label")?)
        } else {
            None
        };
        self.expect_semicolon()?;
        Ok(Stmt::Break(label))
    }

    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Continue, "continue")?;
        let label = if let TokenKind::Identifier(_) = self.peek_kind() {
            Some(self.expect_identifier("continue label")?)
        } else {
            None
        };
        self.expect_semicolon()?;
        Ok(Stmt::Continue(label))
    }

    // parse statements until we hit one of the terminator tokens.
    // the terminator is left unconsumed - caller handles it.
    fn parse_block(&mut self, terminators: &[TokenKind]) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        loop {
            if self.is_at_end() {
                return Err(ParseError::UnexpectedEof { context: "block body".into() });
            }
            if terminators.iter().any(|t| self.check(t)) { break; }
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_end_when(&mut self) -> Result<Option<EndWhen>, ParseError> {
        if !self.check(&TokenKind::When) { return Ok(None); }
        self.advance();
        self.expect(TokenKind::LParen, "(")?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RParen, ")")?;
        self.expect(TokenKind::Colon, ":")?;
        let value = self.parse_expression()?;
        self.expect_semicolon()?;
        Ok(Some(EndWhen { condition, value }))
    }

    fn try_parse_label(&mut self) -> Option<String> {
        // labels look like `name:` before a loop variable
        if let TokenKind::Identifier(name) = self.peek_kind().clone() {
            if self.current + 1 < self.tokens.len()
                && self.tokens[self.current + 1].kind == TokenKind::Colon
            {
                self.advance();
                self.advance();
                return Some(name);
            }
        }
        None
    }

    // Expression parsing - precedence layers from lowest to highest:
    //   or → and → bitwise_or → bitwise_and → equality → comparison
    //   → shift → add → mul → pow → unary → call/index → primary

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_expression_from(&mut self, left: Expr) -> Result<Expr, ParseError> {
        self.parse_binary_from(left, 0)
    }

    fn parse_binary_from(&mut self, mut left: Expr, min_prec: u8) -> Result<Expr, ParseError> {
        loop {
            let (op, prec) = match self.peek_binary_op() {
                Some(p) if p.1 >= min_prec => p,
                _ => break,
            };
            self.advance();
            let rhs_prec = if self.is_right_assoc(&op) { prec } else { prec + 1 };
            let right    = self.parse_unary()?;
            let right    = self.parse_binary_from(right, rhs_prec)?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.check(&TokenKind::PipePipe) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp { left: Box::new(left), op: BinaryOp::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_or()?;
        while self.check(&TokenKind::AmpAmp) {
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expr::BinaryOp { left: Box::new(left), op: BinaryOp::And, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_and()?;
        while self.check(&TokenKind::Pipe) {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expr::BinaryOp { left: Box::new(left), op: BinaryOp::BitOr, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::Amp) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp { left: Box::new(left), op: BinaryOp::BitAnd, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEqEq => BinaryOp::StrictEq,
                TokenKind::EqEq   => BinaryOp::Eq,
                TokenKind::BangEq => BinaryOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_shift()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Less      => BinaryOp::Lt,
                TokenKind::Greater   => BinaryOp::Gt,
                TokenKind::LessEq    => BinaryOp::LtEq,
                TokenKind::GreaterEq => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::LeftShift  => BinaryOp::Shl,
                TokenKind::RightShift => BinaryOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus  => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_exponential()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star    => BinaryOp::Mul,
                TokenKind::Slash   => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_exponential()?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    // right-associative: 2 ** 3 ** 2 == 2 ** (3 ** 2) == 512
    fn parse_exponential(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_unary()?;
        if self.check(&TokenKind::StarStar) {
            self.advance();
            let exp = self.parse_exponential()?;
            return Ok(Expr::BinaryOp { left: Box::new(base), op: BinaryOp::Pow, right: Box::new(exp) });
        }
        Ok(base)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            TokenKind::Bang => {
                self.advance();
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(self.parse_unary()?) })
            }
            TokenKind::Minus => {
                self.advance();
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(self.parse_unary()?) })
            }
            _ => self.parse_call_or_index(),
        }
    }

    fn parse_call_or_index(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::At) {
            self.advance();
            let name = self.expect_call_name()?;
            self.expect(TokenKind::LParen, "(")?;
            let args = self.parse_call_args()?;
            self.expect(TokenKind::RParen, ")")?;
            let mut expr = Expr::Call { name, args };

            // handle @getArr()[0] style chained indexing
            while self.check(&TokenKind::LBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket, "]")?;
                expr = Expr::Index { collection: Box::new(expr), index: Box::new(index) };
            }
            return Ok(expr);
        }

        let mut expr = self.parse_primary()?;

        while self.check(&TokenKind::LBracket) {
            self.advance();
            let index = self.parse_expression()?;
            self.expect(TokenKind::RBracket, "]")?;
            expr = Expr::Index { collection: Box::new(expr), index: Box::new(index) };
        }

        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.check(&TokenKind::RParen) { return Ok(args); }
        loop {
            args.push(self.parse_expression()?);
            if !self.check(&TokenKind::Comma) { break; }
            self.advance();
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.advance();
        match tok.kind.clone() {
            TokenKind::IntLiteral(n)    => Ok(Expr::IntLit(n)),
            TokenKind::LongLiteral(n)   => Ok(Expr::LongLit(n)),
            TokenKind::FloatLiteral(f)  => Ok(Expr::FloatLit(f)),
            TokenKind::StringLiteral(s) => Ok(Expr::StringLit(s)),
            TokenKind::CharLiteral(c)   => Ok(Expr::CharLit(c)),
            TokenKind::BoolLiteral(b)   => Ok(Expr::BoolLit(b)),
            TokenKind::Null             => Ok(Expr::NullLit),
            TokenKind::Identifier(name) => Ok(Expr::Ident(name)),

            TokenKind::LParen => {
                let inner = self.parse_expression()?;
                self.expect(TokenKind::RParen, ")")?;
                Ok(Expr::Grouped(Box::new(inner)))
            }

            TokenKind::LBracket => {
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(&TokenKind::Comma) { break; }
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket, "]")?;
                Ok(Expr::ArrayLit(elements))
            }

            TokenKind::LBrace => {
                // Could be set literal {1, 2, 3} or dict literal {"key": value}
                // Empty braces {} → empty set
                if self.check(&TokenKind::RBrace) {
                    self.advance();
                    return Ok(Expr::SetLit(Vec::new()));
                }

                // Parse first expression, then look ahead
                let first = self.parse_expression()?;

                if self.check(&TokenKind::Colon) {
                    // Dictionary: {"key": value, ...}
                    self.advance(); // consume ':'
                    let first_val = self.parse_expression()?;
                    let mut pairs = vec![(first, first_val)];
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RBrace) { break; }
                        let k = self.parse_expression()?;
                        self.expect(TokenKind::Colon, ":")?;
                        let v = self.parse_expression()?;
                        pairs.push((k, v));
                    }
                    self.expect(TokenKind::RBrace, "}")?;
                    Ok(Expr::DictLit(pairs))
                } else {
                    // Set: {1, 2, 3}
                    let mut elements = vec![first];
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RBrace) { break; }
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(TokenKind::RBrace, "}")?;
                    Ok(Expr::SetLit(elements))
                }
            }

            other => Err(ParseError::UnexpectedToken {
                expected: "expression".into(),
                found:    format!("{:?}", other),
                line:     tok.span.line,
                column:   tok.span.column,
            }),
        }
    }

    fn peek_binary_op(&self) -> Option<(BinaryOp, u8)> {
        match self.peek_kind() {
            TokenKind::PipePipe   => Some((BinaryOp::Or,       1)),
            TokenKind::AmpAmp     => Some((BinaryOp::And,      2)),
            TokenKind::Pipe       => Some((BinaryOp::BitOr,    3)),
            TokenKind::Amp        => Some((BinaryOp::BitAnd,   4)),
            TokenKind::EqEqEq     => Some((BinaryOp::StrictEq, 5)),
            TokenKind::EqEq       => Some((BinaryOp::Eq,       5)),
            TokenKind::BangEq     => Some((BinaryOp::NotEq,    5)),
            TokenKind::Less       => Some((BinaryOp::Lt,       6)),
            TokenKind::Greater    => Some((BinaryOp::Gt,       6)),
            TokenKind::LessEq     => Some((BinaryOp::LtEq,     6)),
            TokenKind::GreaterEq  => Some((BinaryOp::GtEq,     6)),
            TokenKind::LeftShift  => Some((BinaryOp::Shl,      7)),
            TokenKind::RightShift => Some((BinaryOp::Shr,      7)),
            TokenKind::Plus       => Some((BinaryOp::Add,      8)),
            TokenKind::Minus      => Some((BinaryOp::Sub,      8)),
            TokenKind::Star       => Some((BinaryOp::Mul,      9)),
            TokenKind::Slash      => Some((BinaryOp::Div,      9)),
            TokenKind::Percent    => Some((BinaryOp::Mod,      9)),
            TokenKind::StarStar   => Some((BinaryOp::Pow,     10)),
            _ => None,
        }
    }

    fn is_right_assoc(&self, op: &BinaryOp) -> bool {
        matches!(op, BinaryOp::Pow)
    }

    fn parse_type(&mut self) -> Result<CyType, ParseError> {
        let tok = self.advance();
        match tok.kind.clone() {
            TokenKind::Int     => Ok(CyType::Int),
            TokenKind::Long    => Ok(CyType::Long),
            TokenKind::Float   => Ok(CyType::Float),
            TokenKind::String_ => Ok(CyType::StringType),
            TokenKind::Char    => Ok(CyType::Char),
            TokenKind::Bool    => Ok(CyType::Bool),
            TokenKind::Null    => Ok(CyType::Null),
            TokenKind::Set     => Ok(CyType::Set(self.try_parse_generic_arg()?.map(Box::new))),
            TokenKind::Dic     => Ok(CyType::Dic(self.try_parse_generic_arg()?.map(Box::new), None)),
            TokenKind::Arr     => Ok(CyType::Arr(self.try_parse_generic_arg()?.map(Box::new))),
            TokenKind::Identifier(name) => Ok(CyType::Unknown(name)),
            other => Err(ParseError::UnexpectedToken {
                expected: "type name".into(),
                found:    format!("{:?}", other),
                line:     tok.span.line,
                column:   tok.span.column,
            }),
        }
    }

    fn try_parse_generic_arg(&mut self) -> Result<Option<CyType>, ParseError> {
        if self.check(&TokenKind::Less) {
            self.advance();
            let ty = self.parse_type()?;
            self.expect(TokenKind::Greater, ">")?;
            Ok(Some(ty))
        } else {
            Ok(None)
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens[self.current].kind.clone()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.tokens[self.current].kind == kind
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.current].clone();
        if self.current + 1 < self.tokens.len() { self.current += 1; }
        tok
    }

    fn expect(&mut self, expected: TokenKind, name: &str) -> Result<Token, ParseError> {
        if self.check(&expected) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(ParseError::UnexpectedToken {
                expected: format!("`{}`", name),
                found:    format!("{:?}", tok.kind),
                line:     tok.span.line,
                column:   tok.span.column,
            })
        }
    }

    fn expect_semicolon(&mut self) -> Result<(), ParseError> {
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(())
    }

    fn expect_identifier(&mut self, context: &str) -> Result<String, ParseError> {
        let tok = self.peek().clone();
        if let TokenKind::Identifier(name) = tok.kind {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("identifier ({})", context),
                found:    format!("{:?}", tok.kind),
                line:     tok.span.line,
                column:   tok.span.column,
            })
        }
    }

    // write and writeln are keywords in the lexer but callable like any function,
    // so we need to accept them here in addition to plain identifiers
    fn expect_call_name(&mut self) -> Result<String, ParseError> {
        let tok = self.peek().clone();
        let name = match &tok.kind {
            TokenKind::Identifier(n) => n.clone(),
            TokenKind::Read          => "read".into(),
            TokenKind::Write         => "write".into(),
            TokenKind::Writeln       => "writeln".into(),
            _ => return Err(ParseError::UnexpectedToken {
                expected: "function name".into(),
                found:    format!("{:?}", tok.kind),
                line:     tok.span.line,
                column:   tok.span.column,
            }),
        };
        self.advance();
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> Program {
        let tokens = Lexer::new(src).tokenize().expect("lex error");
        Parser::new(tokens).parse().expect("parse error")
    }

    #[test]
    fn test_var_decl() {
        let prog = parse("let x: int = 42;");
        assert!(matches!(&prog.body[0], Stmt::VarDecl { kind: VarKind::Let, .. }));
    }

    #[test]
    fn test_function_decl() {
        let prog = parse("fun add(a: int, b: int): int then\n    return a + b;\nendfun");
        assert!(matches!(&prog.body[0], Stmt::FunDecl { name, .. } if name == "add"));
    }

    #[test]
    fn test_if_else() {
        let prog = parse("if x > 10 then @writeln(\"big\"); else @writeln(\"small\"); endif");
        assert!(matches!(&prog.body[0], Stmt::If { else_body: Some(_), .. }));
    }

    #[test]
    fn test_for_range() {
        let prog = parse("for i from 0 to 10 then @writeln(i); endfor");
        assert!(matches!(&prog.body[0], Stmt::ForRange { var, .. } if var == "i"));
    }

    #[test]
    fn test_operator_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let prog = parse("2 + 3 * 4;");
        if let Stmt::ExprStmt(Expr::BinaryOp { op: BinaryOp::Add, right, .. }) = &prog.body[0] {
            assert!(matches!(right.as_ref(), Expr::BinaryOp { op: BinaryOp::Mul, .. }));
        } else {
            panic!("wrong ast shape");
        }
    }

    #[test]
    fn test_nested_call() {
        let prog = parse(r#"@writeln(@greet("World"));"#);
        assert!(matches!(&prog.body[0], Stmt::ExprStmt(Expr::Call { name, .. }) if name == "writeln"));
    }

    #[test]
    fn test_while() {
        let prog = parse("while count > 0 then count -= 1; endwhile");
        assert!(matches!(&prog.body[0], Stmt::While { .. }));
    }
}
