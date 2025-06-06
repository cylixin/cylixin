use crate::token::{Token, TokenKind};
use std::collections::HashMap;

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    current_char: Option<char>,
    start_index: usize,
    current_index: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer {
            source,
            chars: source.chars().peekable(),
            current_char: None,
            start_index: 0,
            current_index: 0,
            line: 1,
            column: 0,
            keywords: Self::build_keywords_map(),
        };
        lexer.advance();
        lexer
    }

    fn build_keywords_map() -> HashMap<String, TokenKind> {
        let mut map = HashMap::new();
        map.insert("let".to_string(), TokenKind::Let);
        map.insert("var".to_string(), TokenKind::Var);
        map.insert("const".to_string(), TokenKind::Const);
        map.insert("if".to_string(), TokenKind::If);
        map.insert("else".to_string(), TokenKind::Else);
        map.insert("elif".to_string(), TokenKind::Elif);
        map.insert("for".to_string(), TokenKind::For);
        map.insert("while".to_string(), TokenKind::While);
        map.insert("fun".to_string(), TokenKind::Fun);
        map.insert("return".to_string(), TokenKind::Return);
        map.insert("then".to_string(), TokenKind::Then);
        map.insert("endfun".to_string(), TokenKind::EndFun);
        map.insert("endif".to_string(), TokenKind::EndIf);
        map.insert("endfor".to_string(), TokenKind::EndFor);
        map.insert("endwhile".to_string(), TokenKind::EndWhile);
        map.insert("true".to_string(), TokenKind::True);
        map.insert("false".to_string(), TokenKind::False);
        map.insert("null".to_string(), TokenKind::Null);
        map.insert("int".to_string(), TokenKind::IntType);
        map.insert("float".to_string(), TokenKind::FloatType);
        map.insert("strg".to_string(), TokenKind::StrgType);
        map.insert("bool".to_string(), TokenKind::BoolType);
        map.insert("set".to_string(), TokenKind::SetType);
        map.insert("dic".to_string(), TokenKind::DicType);
        // map.insert("!empty".to_string(), TokenKind::NotEmpty); // Will handle manually
        map.insert("empty".to_string(), TokenKind::Empty);
        map.insert("break".to_string(), TokenKind::Break);
        map.insert("and".to_string(), TokenKind::And);
        map.insert("or".to_string(), TokenKind::Or);
        map.insert("not".to_string(), TokenKind::Not);
        map.insert("NaN".to_string(), TokenKind::NaN);
        map
    }

    fn advance(&mut self) {
        self.current_char = self.chars.next();
        if let Some(c) = self.current_char {
            self.current_index += c.len_utf8();
            self.column += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn match_char(&mut self, expected_char: char) -> bool {
        if self.current_char == Some(expected_char) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                self.advance();
            } else {
                break;
            }
        }
    }

    fn handle_single_line_comment(&mut self) {
        while let Some(c) = self.current_char {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn handle_multi_line_comment(&mut self) -> Result<(), String> {
        let comment_start_line = self.line;
        let comment_start_col = self.column;

        self.advance();

        loop {
            match self.current_char {
                Some('*') => {
                    self.advance();
                    if let Some('/') = self.current_char {
                        self.advance();
                        return Ok(());
                    }
                },
                Some('\n') => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                },
                Some(_) => {
                    self.advance();
                },
                None => {
                    return Err(format!("Unterminated multi-line comment starting at line {} column {}", comment_start_line, comment_start_col));
                }
            }
        }
    }

    fn read_number(&mut self, token_start_line: usize, token_start_column: usize) -> Token {
        while let Some(c) = self.current_char {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let mut is_float = false;
        if let Some('.') = self.current_char {
            if let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    is_float = true;
                    self.advance();
                    while let Some(c_after_dot) = self.current_char {
                        if c_after_dot.is_ascii_digit() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        let lexeme_end_index = if self.current_char.is_some() {
            self.current_index - self.current_char.unwrap().len_utf8()
        } else {
            self.current_index
        };

        let lexeme = self.source[self.start_index..lexeme_end_index].to_string();

        if is_float {
            match lexeme.parse::<f64>() {
                Ok(val) => Token::new(TokenKind::Float(val), lexeme, token_start_line, token_start_column),
                Err(_) => Token::new(TokenKind::Error(format!("Invalid float literal: {}", lexeme)), lexeme, token_start_line, token_start_column),
            }
        } else {
            match lexeme.parse::<i64>() {
                Ok(val) => Token::new(TokenKind::Integer(val), lexeme, token_start_line, token_start_column),
                Err(_) => Token::new(TokenKind::Error(format!("Invalid integer literal: {}", lexeme)), lexeme, token_start_line, token_start_column),
            }
        }
    }

    fn read_identifier_or_keyword(&mut self, token_start_line: usize, token_start_column: usize) -> Token {
        while let Some(c) = self.current_char {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme_end_index = if self.current_char.is_some() {
            self.current_index - self.current_char.unwrap().len_utf8()
        } else {
            self.current_index
        };

        let lexeme = self.source[self.start_index..lexeme_end_index].to_string();

        if let Some(kind) = self.keywords.get(&lexeme) {
            Token::new(kind.clone(), lexeme, token_start_line, token_start_column)
        } else {
            Token::new(TokenKind::Identifier(lexeme.clone()), lexeme, token_start_line, token_start_column)
        }
    }

    fn read_string(&mut self, token_start_line: usize, token_start_column: usize) -> Token {
        // `self.current_char` is the opening quote when this function is called.
        self.advance(); // Consume the opening quote. Now current_char is the first char of content.

        let mut string_content = String::new();
        let mut escaped = false;

        loop {
            match self.current_char {
                Some('"') if !escaped => {
                    self.advance(); // Consume the closing quote.
                    // `self.start_index` was set to the opening quote's position by next_token.
                    // `self.current_index` is now past the closing quote.
                    // So, slicing from `self.start_index` to `self.current_index` captures the whole lexeme.
                    let full_lexeme = self.source[self.start_index..self.current_index].to_string();
                    return Token::new(TokenKind::String(string_content), full_lexeme, token_start_line, token_start_column);
                },
                Some('\\') if !escaped => {
                    escaped = true;
                    self.advance();
                },
                Some('\n') => {
                    return Token::new(TokenKind::Error("Unterminated string literal (newline found)".to_string()),
                                      string_content, token_start_line, token_start_column);
                },
                Some(c) => {
                    if escaped {
                        match c {
                            'n' => string_content.push('\n'),
                            't' => string_content.push('\t'),
                            '\\' => string_content.push('\\'),
                            '"' => string_content.push('"'),
                            _ => {
                                return Token::new(TokenKind::Error(format!("Invalid escape sequence: \\{}", c)),
                                                  format!("\\{}", c), token_start_line, token_start_column);
                            }
                        }
                        escaped = false;
                    } else {
                        string_content.push(c);
                    }
                    self.advance();
                },
                None => {
                    return Token::new(TokenKind::Error("Unterminated string literal (EOF)".to_string()),
                                      string_content, token_start_line, token_start_column);
                }
            }
        }
    }


    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            match self.current_char {
                Some('/') => {
                    match self.peek() {
                        Some('/') => {
                            self.advance();
                            self.advance();
                            self.handle_single_line_comment();
                            continue;
                        },
                        Some('*') => {
                            self.advance();
                            if let Err(msg) = self.handle_multi_line_comment() {
                                return Token::new(TokenKind::Error(msg), "".to_string(), self.line, self.column);
                            }
                            continue;
                        },
                        _ => {
                            break;
                        }
                    }
                },
                _ => break,
            }
        }

        self.start_index = self.current_index - self.current_char.map_or(0, |c| c.len_utf8());
        let token_start_line = self.line;
        let token_start_column = self.column;

        let char_to_match = self.current_char;

        match char_to_match {
            None => {
                Token::new(TokenKind::EOF, "".to_string(), token_start_line, token_start_column)
            },

            // Single-character tokens
            Some('(') => { self.advance(); Token::new(TokenKind::LParen, "(".to_string(), token_start_line, token_start_column) },
            Some(')') => { self.advance(); Token::new(TokenKind::RParen, ")".to_string(), token_start_line, token_start_column) },
            Some('{') => { self.advance(); Token::new(TokenKind::LBrace, "{".to_string(), token_start_line, token_start_column) },
            Some('}') => { self.advance(); Token::new(TokenKind::RBrace, "}".to_string(), token_start_line, token_start_column) },
            Some('[') => { self.advance(); Token::new(TokenKind::LBracket, "[".to_string(), token_start_line, token_start_column) },
            Some(']') => { self.advance(); Token::new(TokenKind::RBracket, "]".to_string(), token_start_line, token_start_column) },
            Some(',') => { self.advance(); Token::new(TokenKind::Comma, ",".to_string(), token_start_line, token_start_column) },
            Some(';') => { self.advance(); Token::new(TokenKind::Semicolon, ";".to_string(), token_start_line, token_start_column) },
            Some(':') => { self.advance(); Token::new(TokenKind::Colon, ":".to_string(), token_start_line, token_start_column) },
            Some('.') => { self.advance(); Token::new(TokenKind::Dot, ".".to_string(), token_start_line, token_start_column) },
            Some('@') => { self.advance(); Token::new(TokenKind::At, "@".to_string(), token_start_line, token_start_column) },

            // Multi-character operators and `!empty`
            Some('=') => {
                let kind;
                if self.peek() == Some('=') {
                    self.advance(); // Consume first '='
                    self.advance(); // Consume second '='
                    if self.peek() == Some('=') {
                        self.advance(); // Consume third '='
                        kind = TokenKind::StrictEq;
                    } else {
                        kind = TokenKind::EqEq;
                    }
                } else {
                    self.advance(); // Only consume the single '='
                    kind = TokenKind::Eq;
                }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('!') => {
                let kind;
                let lexeme;

                // Check for `!empty` keyword *first*
                // Peek ahead to see if "empty" follows '!'
                let remaining_source_after_bang_start = self.current_index - self.current_char.map_or(0, |c| c.len_utf8());
                let remaining_source_slice = &self.source[remaining_source_after_bang_start..];

                if remaining_source_slice.starts_with("!empty") {
                    self.advance(); // Consume '!'
                    // Consume "empty"
                    for _ in 0.."empty".len() {
                        self.advance();
                    }
                    kind = TokenKind::NotEmpty;
                    lexeme = "!empty".to_string(); // Manually set lexeme
                } else if self.peek() == Some('=') { // Check for '!='
                    self.advance(); // Consume '!'
                    self.advance(); // Consume '='
                    kind = TokenKind::BangEq;
                    lexeme = "!=".to_string();
                } else {
                    self.advance(); // Consume '!' (the erroneous character)
                    return Token::new(TokenKind::Error("Unexpected character: '!'. Expected '!=' or '!empty'".to_string()),
                                      "!".to_string(), token_start_line, token_start_column);
                }
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('<') => {
                let kind;
                if self.peek() == Some('=') {
                    self.advance(); self.advance(); kind = TokenKind::LessEq;
                } else if self.peek() == Some('<') {
                    self.advance(); self.advance(); kind = TokenKind::LeftShift;
                } else { self.advance(); kind = TokenKind::Less; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('>') => {
                let kind;
                if self.peek() == Some('=') {
                    self.advance(); self.advance(); kind = TokenKind::GreaterEq;
                } else if self.peek() == Some('>') {
                    self.advance(); self.advance(); kind = TokenKind::RightShift;
                } else { self.advance(); kind = TokenKind::Greater; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('+') => {
                let kind;
                if self.peek() == Some('=') { self.advance(); self.advance(); kind = TokenKind::PlusEq; }
                else { self.advance(); kind = TokenKind::Plus; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('-') => {
                let kind;
                if self.peek() == Some('=') { self.advance(); self.advance(); kind = TokenKind::MinusEq; }
                else { self.advance(); kind = TokenKind::Minus; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('*') => {
                let kind;
                if self.peek() == Some('*') { self.advance(); self.advance(); kind = TokenKind::DoubleStar; }
                else if self.peek() == Some('=') { self.advance(); self.advance(); kind = TokenKind::StarEq; }
                else { self.advance(); kind = TokenKind::Star; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('%') => {
                let kind;
                if self.peek() == Some('=') { self.advance(); self.advance(); kind = TokenKind::PercentEq; }
                else { self.advance(); kind = TokenKind::Percent; }
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(kind, lexeme, token_start_line, token_start_column)
            },
            Some('/') => {
                self.advance(); // Consume the '/'
                let lexeme = self.source[self.start_index..self.current_index].to_string();
                Token::new(TokenKind::Slash, lexeme, token_start_line, token_start_column)
            },

            // Literals (numbers and strings) and Identifiers/Keywords.
            Some(c) if c.is_ascii_digit() => {
                self.read_number(token_start_line, token_start_column)
            },
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.read_identifier_or_keyword(token_start_line, token_start_column)
            },
            Some('"') => {
                // Do NOT self.advance() here. read_string will consume the opening quote.
                self.read_string(token_start_line, token_start_column)
            },

            // Unrecognized character
            Some(c) => {
                let err_msg = format!("Unexpected character: '{}'", c);
                self.advance();
                let lexeme = c.to_string();
                Token::new(TokenKind::Error(err_msg), lexeme, token_start_line, token_start_column)
            },
        }
    }
}