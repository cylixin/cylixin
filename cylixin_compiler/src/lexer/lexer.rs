use super::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    UnexpectedChar     { ch: char, line: usize, column: usize },
    UnterminatedString { line: usize, column: usize },
    InvalidCharLiteral { line: usize, column: usize },
    UnterminatedBlockComment { line: usize, column: usize },
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::UnexpectedChar { ch, line, column } =>
                write!(f, "Unexpected character '{}' at {}:{}", ch, line, column),
            LexerError::UnterminatedString { line, column } =>
                write!(f, "Unterminated string at {}:{}", line, column),
            LexerError::InvalidCharLiteral { line, column } =>
                write!(f, "Invalid char literal at {}:{}", line, column),
            LexerError::UnterminatedBlockComment { line, column } =>
                write!(f, "Unterminated block comment opened at {}:{}", line, column),
        }
    }
}

pub struct Lexer {
    // Vec<char> instead of &str so indexing always gives a full Unicode char
    source:  Vec<char>,
    current: usize,
    line:    usize,
    column:  usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer { source: source.chars().collect(), current: 0, line: 1, column: 1 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments()?;
            if self.is_at_end() {
                tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
                break;
            }
            tokens.push(self.scan_token()?);
        }
        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Token, LexerError> {
        let line = self.line;
        let col  = self.column;
        let ch   = self.advance();

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '@' => TokenKind::At,

            '+' => if self.match_next('=') { TokenKind::PlusEq  } else { TokenKind::Plus  },
            '-' => if self.match_next('=') { TokenKind::MinusEq } else { TokenKind::Minus },
            '/' => if self.match_next('=') { TokenKind::SlashEq } else { TokenKind::Slash },
            '%' => if self.match_next('=') { TokenKind::PercentEq } else { TokenKind::Percent },

            // order matters here: check ** before *, and **= before **
            '*' => {
                if self.match_next('*') {
                    if self.match_next('=') { TokenKind::StarStarEq }
                    else { TokenKind::StarStar }
                } else if self.match_next('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }

            // same idea for ===  vs  ==
            '=' => {
                if self.match_next('=') {
                    if self.match_next('=') { TokenKind::EqEqEq } else { TokenKind::EqEq }
                } else {
                    TokenKind::Eq
                }
            }

            '!' => if self.match_next('=') { TokenKind::BangEq } else { TokenKind::Bang },

            '>' => {
                if self.match_next('=')      { TokenKind::GreaterEq   }
                else if self.match_next('>') { TokenKind::RightShift  }
                else                         { TokenKind::Greater     }
            }
            '<' => {
                if self.match_next('=')      { TokenKind::LessEq    }
                else if self.match_next('<') { TokenKind::LeftShift }
                else                         { TokenKind::Less      }
            }

            '&' => if self.match_next('&') { TokenKind::AmpAmp  } else { TokenKind::Amp  },
            '|' => if self.match_next('|') { TokenKind::PipePipe } else { TokenKind::Pipe },

            '"'  => self.lex_string(line, col)?,
            '\'' => self.lex_char(line, col)?,

            c if c.is_ascii_digit()              => self.lex_number(c)?,
            c if c.is_alphabetic() || c == '_'   => self.lex_word(c),

            c => return Err(LexerError::UnexpectedChar { ch: c, line, column: col }),
        };

        Ok(Token::new(kind, line, col))
    }

    fn lex_string(&mut self, start_line: usize, start_col: usize) -> Result<TokenKind, LexerError> {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' {
                if self.is_at_end() {
                    return Err(LexerError::UnterminatedString { line: start_line, column: start_col });
                }
                match self.advance() {
                    'n'  => value.push('\n'),
                    't'  => value.push('\t'),
                    'r'  => value.push('\r'),
                    '"'  => value.push('"'),
                    '\\' => value.push('\\'),
                    '0'  => value.push('\0'),
                    // unknown escape - keep it as-is rather than hard erroring,
                    // semantic analysis can catch it with a better message
                    other => { value.push('\\'); value.push(other); }
                }
            } else {
                value.push(ch);
            }
        }

        if self.is_at_end() {
            return Err(LexerError::UnterminatedString { line: start_line, column: start_col });
        }
        self.advance(); // closing "
        Ok(TokenKind::StringLiteral(value))
    }

    fn lex_char(&mut self, start_line: usize, start_col: usize) -> Result<TokenKind, LexerError> {
        if self.is_at_end() {
            return Err(LexerError::InvalidCharLiteral { line: start_line, column: start_col });
        }

        let ch = self.advance();
        let value = if ch == '\\' {
            if self.is_at_end() {
                return Err(LexerError::InvalidCharLiteral { line: start_line, column: start_col });
            }
            match self.advance() {
                'n'  => '\n',
                't'  => '\t',
                'r'  => '\r',
                '\'' => '\'',
                '\\' => '\\',
                '0'  => '\0',
                _    => return Err(LexerError::InvalidCharLiteral { line: start_line, column: start_col }),
            }
        } else {
            ch
        };

        if self.is_at_end() || self.peek() != '\'' {
            return Err(LexerError::InvalidCharLiteral { line: start_line, column: start_col });
        }
        self.advance(); // closing '
        Ok(TokenKind::CharLiteral(value))
    }

    fn lex_number(&mut self, first: char) -> Result<TokenKind, LexerError> {
        let mut num = String::from(first);

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num.push(self.advance());
        }

        // float: only treat the dot as decimal if a digit follows it
        // otherwise  `5.endfor` would misread the dot
        if !self.is_at_end()
            && self.peek() == '.'
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            num.push(self.advance()); // the dot
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                num.push(self.advance());
            }
            return Ok(TokenKind::FloatLiteral(num.parse().unwrap_or(0.0)));
        }

        // L/l suffix = long integer
        if !self.is_at_end() && (self.peek() == 'L' || self.peek() == 'l') {
            self.advance();
            return Ok(TokenKind::LongLiteral(num.parse().unwrap_or(0)));
        }

        Ok(TokenKind::IntLiteral(num.parse().unwrap_or(0)))
    }

    fn lex_word(&mut self, first: char) -> TokenKind {
        let mut word = String::from(first);
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            word.push(self.advance());
        }
        // keyword or identifier - the token table decides
        TokenKind::from_keyword(&word).unwrap_or(TokenKind::Identifier(word))
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexerError> {
        loop {
            if self.is_at_end() { break; }
            match self.peek() {
                ' ' | '\t' | '\r' | '\n' => { self.advance(); }
                '/' => match self.peek_next() {
                    Some('/') => {
                        // line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    }
                    Some('*') => {
                        // block comment - track where it opened for the error message
                        let (sl, sc) = (self.line, self.column);
                        self.advance(); self.advance(); // consume /*
                        loop {
                            if self.is_at_end() {
                                return Err(LexerError::UnterminatedBlockComment {
                                    line: sl, column: sc,
                                });
                            }
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); self.advance(); // consume */
                                break;
                            }
                            self.advance();
                        }
                    }
                    _ => break, // just a / operator, stop skipping
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        if ch == '\n' { self.line += 1; self.column = 1; }
        else          { self.column += 1; }
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).copied()
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected { return false; }
        self.current += 1;
        self.column  += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenKind;

    fn kinds(src: &str) -> Vec<TokenKind> {
        Lexer::new(src).tokenize().expect("lex error").into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_variable_declaration() {
        assert_eq!(kinds("let x: int = 42;"), vec![
            TokenKind::Let,
            TokenKind::Identifier("x".into()),
            TokenKind::Colon,
            TokenKind::Int,
            TokenKind::Eq,
            TokenKind::IntLiteral(42),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_function_call_with_at() {
        let result = kinds(r#"@greet("World")"#);
        assert_eq!(result, vec![
            TokenKind::At,
            TokenKind::Identifier("greet".into()),
            TokenKind::LParen,
            TokenKind::StringLiteral("World".into()),
            TokenKind::RParen,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_exponentiation_operator() {
        assert!(kinds("x ** 2").contains(&TokenKind::StarStar));
        assert!(kinds("x **= 3").contains(&TokenKind::StarStarEq));
    }

    #[test]
    fn test_strict_equality() {
        assert!(kinds("a === b").contains(&TokenKind::EqEqEq));
        assert!(kinds("a == b").contains(&TokenKind::EqEq));
    }

    #[test]
    fn test_for_range_syntax() {
        assert_eq!(kinds("for i from 0 to 5 then"), vec![
            TokenKind::For,
            TokenKind::Identifier("i".into()),
            TokenKind::From,
            TokenKind::IntLiteral(0),
            TokenKind::To,
            TokenKind::IntLiteral(5),
            TokenKind::Then,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_block_comment_skipped() {
        assert_eq!(kinds("let /* comment */ x = 5;"), vec![
            TokenKind::Let,
            TokenKind::Identifier("x".into()),
            TokenKind::Eq,
            TokenKind::IntLiteral(5),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_long_and_float_literals() {
        assert!(matches!(kinds("1234L")[0], TokenKind::LongLiteral(1234)));
        assert!(matches!(kinds("3.14")[0],  TokenKind::FloatLiteral(_)));
    }

    #[test]
    fn test_unterminated_string() {
        assert!(matches!(
            Lexer::new(r#""hello"#).tokenize(),
            Err(LexerError::UnterminatedString { .. })
        ));
    }

    #[test]
    fn test_line_tracking() {
        let tokens = Lexer::new("let x = 1;\nlet y = 2;").tokenize().unwrap();
        let y = tokens.iter().find(|t| t.kind == TokenKind::Identifier("y".into())).unwrap();
        assert_eq!(y.span.line, 2);
    }
}
