

// It defines all the possible kinds (types) of tokens that our lexer can recognize.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let, Var, Const,
    If, Else, Elif, 
    For, While, Fun, Then, Break, Continue, Return,
    EndFun, EndIf, EndFor, EndWhile, EndElse, EndElif,
    True, False, Null,
    IntType, FloatType, StrgType, BoolType, SetType, DicType, // Type keywords
    Empty, NotEmpty,

    // Operators
    Plus, Minus, Star, Slash, Percent, DoubleStar, DoubleSlash,                   // Arithemetic: +, -, *, /, %, **, //
    EqEq, StrictEq, BangEq, Greater, Less, GreaterEq, LessEq,                     // Comparison: ==, ===, !=, >, <, >=, <=
    And, Or, Not,                                                                 // Logical: and, or, not
    Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq, DoubleStarEq, DoubleSlashEq, // Assignment: =, +=, -=, *=, /=, %=, **=, //=
    RightShift, LeftShift,                                                        // Bitwise: >>, <<

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    NaN,

    // Punctuators
    LParen, RParen,                   // ( )
    LBrace, RBrace,                   // { }
    LBracket, RBracket,               // [ ]
    Comma, Semicolon, Colon, Dot, At, // , ; : . @

    // Identifier
    Identifier(String),

    // End of file
    EOF,  // End Of File: Indicates the end of the input source code.

    // Error Token
    Error(String), // Contains an error message
}


// This struct represents a single token found by the lexer.
// It holds the kind of token, its actual text (lexeme), and its position in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind, // The Token Type (e.g., Keyword, Identifier, Literal, etc.)
    pub lexeme: String,  // The actual text that formed the token
    pub line: usize,     // The line number where the token starts
    pub column: usize,   // The column number where the token starts
}

impl Token {
    // A constructor for creating new Token instances
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Token {kind, lexeme, line, column}
    }
}