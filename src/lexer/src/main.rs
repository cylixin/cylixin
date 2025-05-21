use std::io;

#[derive(Debug)]
enum TokenType {
    Keyword(String),            //keyword
    Identifier(String),         //Identifier
    Operator(String),           //Operator
    // Punctuation(String),        //Punctuation
    // Comment(String),            //Comment
    // Whitespace(String),         //Whitespace
    StringLiteral(String),      //String literal
    IntLiteral(i64),            //Integer literal
    FloatLiteral(f64),          //Float literal
    // CharLiteral(char),          //Character literal
    // BoolLiteral(bool),          //Boolean literal
    Error(String),              //Error
    BraceClose,                 //Closing brace
    BracketOpen,                //Opening bracket
    BraceOpen,                  //Opening brace
    BracketClose,               //Closing bracket
    ParenOpen,                  //Opening parenthesis
    ParenClose,                 //Closing parenthesis
    Semicolon,                  //Semicolon
    Colon,                      //Colon
    Comma,                      //Comma
    Dot,                        //Dot
    // class,                     //Class
}

#[derive(Debug)]
struct Token {
    _type: TokenType,
    line: usize,
    column: usize,
}



fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();        //Iterate with index and peek
    let mut line = 1;                                            //Line number
    let mut column = 0;                                          //Column number

    while let Some((index, current_char)) = chars.next() {
        // println!("Current char: {}, index {}", current_char, index);
        column = index + 1; // Calculate column for the current character

        match current_char {
            ' ' | '\t' => {
                column += 1;  // Increment column for whitespace
            }

            '\n' => {
                line += 1;   // Reset column for new line
                column = 1;  // Reset column for new line
            }

            '{' => tokens.push(Token { _type: TokenType::BraceOpen, line: line, column: column }),

            '}' => tokens.push(Token { _type: TokenType::BraceClose, line: line, column: column }),

            '[' => tokens.push(Token { _type: TokenType::BracketOpen, line: line, column: column }),

            ']' => tokens.push(Token { _type: TokenType::BracketClose, line: line, column: column }),

            '(' => tokens.push(Token { _type: TokenType::ParenOpen, line: line, column: column }),

            ')' => tokens.push(Token { _type: TokenType::ParenClose, line: line, column: column }),

            ';' => tokens.push(Token { _type: TokenType::Semicolon, line: line, column: column }),

            ':' => tokens.push(Token { _type: TokenType::Colon, line: line, column: column }),

            ',' => tokens.push(Token { _type: TokenType::Comma, line: line, column: column }),

            'l' if input[index..].starts_with("let") => {          // Check if the character is 'l' and the word is "let"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("let")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "et"
            }
            'v' if input[index..].starts_with("var") => {          // Check if the character is 'v' and the word is "var"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("var")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "ar"
            }
            'i' if input[index..].starts_with("if") => {           // Check if the character is 'i' and the word is "if"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("if")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(1); // Skip the next of "f"
            }
            'e' if input[index..].starts_with("else") => {         // Check if the character is 'e' and the word is "else"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("else")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "lse"
            } 
            'e' if input[index..].starts_with("elif") => {         // Check if the character is 'e' and the word is "elif"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("elif")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "se"
            }
            'w' if input[index..].starts_with("while") => {       // Check if the character is 'w' and the word is "while"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("while")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(3); // Skip the next of "hile"
            }
            'f' if input[index..].starts_with("for") => {        // Check if the character is 'f' and the word is "for"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("for")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "or"
            }
            'f' if input[index..].starts_with("fun") => {        // Check if the character is 'f' and the word is "function"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("fun")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(2); // Skip the next of "unction"
            }
            'r' if input[index..].starts_with("return") => {      // Check if the character is 'r' and the word is "return"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("return")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(5); // Skip the next of "eturn"
            }
            'b' if input[index..].starts_with("break") => {      // Check if the character is 'b' and the word is "break"
                tokens.push(Token {
                    _type: TokenType::Keyword(String::from("break")),
                    line: 1,
                    column: index + 1
                });
                chars.nth(4); // Skip the next of "reak"
            }


            char if char.is_alphabetic() || char == '_' => {     // Check if the character is alphabetic or underscore
                let start_index = index;
                let mut identifier = String::new();
                identifier.push(char);
                column += 1; // Increment column for 
                
                while let Some(&(next_index, next_char)) = chars.peek() {
                    if next_char.is_alphabetic() || next_char == '_' {
                        identifier.push(chars.next().unwrap().1);
                        column += 1; // Increment column for identifier
                    } else {
                        break;
                    }
                }
                tokens.push(Token { _type: TokenType::Identifier(identifier), line, column: start_index + 1 });
            }

            digit if digit.is_digit(10) => {                     // Check if the character is a digit
                let start_index = index;
                let mut number = String::new();
                number.push(digit);
                let mut is_float = false; // Reset is_float for new number
                column += 1; // Increment column for digit

                while let Some(&(next_index, next_char)) = chars.peek() {
                    if next_char.is_digit(10) {
                        number.push(chars.next().unwrap().1);
                        column += 1; // Increment column for number
                    } else if next_char == '.' {   // Check for decimal point
                        is_float = true; // Set is_float to true if a decimal point is found
                        number.push(chars.next().unwrap().1);
                        column += 1; // Increment column for decimal point
                        if let Some(&(next_after_dot_index, next_after_dot_char)) = chars.peek() {
                            if next_after_dot_char.is_digit(10) {
                                number.push(chars.next().unwrap().1);
                                column += 1; // Increment column for digit after decimal point
                            } else {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }

                if is_float {                     // Check if the number is a float, then push float literal
                    if let Ok(value) = number.parse::<f64>() {
                        tokens.push(Token { _type: TokenType::FloatLiteral(value), line, column: start_index + 1 });
                    } else {
                        println!("Error parsing float {} at line {} and column {}", number, line, column);
                    }
                } else if let Ok(value) = number.parse::<i64>() {   // Check if the number is an integer, then push integer literal
                    tokens.push(Token { _type: TokenType::IntLiteral(value), line, column: start_index + 1 });
                } else {
                    println!("Error parsing integer {} at line {} and column {}", number, line, column);    // Error (Just in case)
                }
            }

            '.' => {
                tokens.push(Token { _type: TokenType::Dot, line: line, column: column });
            }

            '"' | '\'' => {
                let start_index = index;
                let opening_quote_char = current_char;
                let mut string_literal = String::new();
                column += 1;

                // eprintln!("--- DEBUG: Entering string literal lexing ---");
                // eprintln!("  Opening quote: '{}' at line {} column {}", opening_quote_char, line, start_index + 1);

                let mut error_message: Option<String> = None;
                let mut found_closing_quote = false; // NEW FLAG

                while let Some(&(next_index, next_char)) = chars.peek() {
                    // eprintln!("--- DEBUG: PEEKED next_char: '{}' at index {} ({}) ---", next_char, next_index, next_char as u32);

                    if next_char == opening_quote_char {
                        // eprintln!("--- DEBUG: Found matching closing quote '{}'. Breaking. ---", next_char);
                        chars.next(); // Consume the closing quote
                        column += 1;
                        found_closing_quote = true; // Set the flag
                        break;
                    } else if next_char == '\\' {
                        eprintln!("--- DEBUG: Found backslash. Consuming and looking for escaped char. ---");
                        chars.next(); // Consume the backslash itself
                        column += 1;

                        if let Some((_, escaped_char)) = chars.next() {
                            column += 1;
                            eprintln!("--- DEBUG: Escaped char found: '{}' ({}) ---", escaped_char, escaped_char as u32);

                            match escaped_char {
                                'n' => string_literal.push('\n'),
                                't' => string_literal.push('\t'),
                                'r' => string_literal.push('\r'),
                                '\\' => string_literal.push('\\'),
                                '"' => string_literal.push('"'),
                                '\'' => string_literal.push('\''),
                                _ => {
                                    eprintln!("!!! ERROR: Unknown escape sequence '\\{}' at line {} column {}", escaped_char, line, column - 1);
                                    error_message = Some(format!("Unknown escape sequence '\\{}'", escaped_char));
                                    break; // Break the while loop immediately on error
                                }
                            }
                        } else {
                            eprintln!("!!! ERROR: Incomplete escape sequence (backslash at EOF) at line {} column {}", line, column - 1);
                            error_message = Some("Incomplete escape sequence".to_string());
                            break;
                        }
                    } else if next_char == '\n' || next_char == '\r' {
                        eprintln!("!!! ERROR: Unescaped newline in string literal at line {} column {}", line, column);
                        error_message = Some("Unescaped newline in string literal".to_string());
                        break;
                    } else {
                        // eprintln!("--- DEBUG: Pushing regular char: '{}' ---", next_char);
                        string_literal.push(chars.next().unwrap().1);
                        column += 1;
                    }
                }

                // Decide whether to push StringLiteral or Error based on error_message AND found_closing_quote
                if let Some(msg) = error_message { // An internal error occurred during string parsing
                    eprintln!("!!! FINAL DECISION: Pushing Error token: {}. Location: line {} column {}", msg, line, start_index + 1);
                    tokens.push(Token { _type: TokenType::Error(msg), line, column: start_index + 1 });
                } else if !found_closing_quote { // No internal error, but loop finished without finding closing quote
                    eprintln!("!!! FINAL DECISION: Pushing Error token: Unclosed string literal. Location: line {} column {}", line, start_index + 1);
                    tokens.push(Token { _type: TokenType::Error("Unclosed string literal".to_string()), line, column: start_index + 1 });
                } else { // No internal error AND closing quote was found
                    eprintln!("--- FINAL DECISION: Pushing StringLiteral: \"{}\" ---", string_literal);
                    tokens.push(Token { _type: TokenType::StringLiteral(string_literal), line, column: start_index + 1 });
                }
            }


            '+' | '-' | '*' | '/' | '%' | '=' | '|' | '&' | '!' => {
                let start_index = index;
                let operator = current_char.to_string();
                column += 1;

                if let Some(&(next_index, next_char)) = chars.peek() {
                    let two_char_op = format!("{}{}", current_char, next_char);
                    match two_char_op.as_str() {
                        "==" | "!=" | ">=" | "<=" | "||" | "&&" | "+=" | "-=" | "*=" | "/=" | "%=" => {
                            tokens.push(Token { _type: TokenType::Operator(two_char_op), line, column: start_index + 1 });
                            chars.next(); // Consume the next character
                            column += 1;
                        }
                        _ => {
                            tokens.push(Token { _type: TokenType::Operator(operator), line, column: start_index + 1 });
                        }
                    }
                } else {
                    tokens.push(Token { _type: TokenType::Operator(operator), line, column: start_index + 1 });
                }
            }
            
            _ => {
                println!("Found {} at line {} and column {}", current_char, line, column);
            }
        }
    }

    tokens
}


fn main() {
    loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let tokens = lex(&input);
        println!("Tokens: {:?}", tokens);
        // Process the input
    }
}
