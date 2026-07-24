#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Token { kind, span: Span::new(line, column) }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TokenKind {
    // literals
    IntLiteral(i64),
    LongLiteral(i64),   // 42L suffix
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    Identifier(String),

    // declaration keywords
    Let,
    Var,
    Const,

    // types
    Int,
    Long,
    Float,
    String_,  // can't name this String in Rust, so String_ internally
    Char,
    Bool,
    Set,
    Dic,
    Arr,
    Null,

    // control flow
    If,
    Else,
    Elif,
    Then,
    When,

    // loops
    For,
    From,
    To,
    While,
    Break,
    Continue,

    // functions
    Fun,
    Return,

    // block terminators - each block type gets its own so you always
    // know what's closing at a glance
    EndIf,
    EndFor,
    EndWhile,
    EndFun,

    True,
    False,
    Empty,

    // built-in i/o, called with @ like any other function
    Read,
    Write,
    Writeln,

    // arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,    // **

    // comparison
    EqEqEq,     // === strict equality (value + type)
    EqEq,       // ==
    BangEq,     // !=
    Greater,
    Less,
    GreaterEq,
    LessEq,

    // logical
    AmpAmp,
    PipePipe,
    Bang,

    // assignment
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    StarStarEq,

    // bitwise - reserved for later
    Amp,
    Pipe,
    RightShift,
    LeftShift,

    // delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    At,     // @ prefix on all function calls

    Eof,
}

impl TokenKind {
    /// Returns Some(keyword) if the word is reserved, None if it's a user identifier.
    pub fn from_keyword(word: &str) -> Option<TokenKind> {
        match word {
            "let"      => Some(TokenKind::Let),
            "var"      => Some(TokenKind::Var),
            "const"    => Some(TokenKind::Const),
            "int"      => Some(TokenKind::Int),
            "long"     => Some(TokenKind::Long),
            "float"    => Some(TokenKind::Float),
            "string"   => Some(TokenKind::String_),
            "char"     => Some(TokenKind::Char),
            "bool"     => Some(TokenKind::Bool),
            "set"      => Some(TokenKind::Set),
            "dic"      => Some(TokenKind::Dic),
            "arr"      => Some(TokenKind::Arr),
            "null"     => Some(TokenKind::Null),
            "if"       => Some(TokenKind::If),
            "else"     => Some(TokenKind::Else),
            "elif"     => Some(TokenKind::Elif),
            "then"     => Some(TokenKind::Then),
            "when"     => Some(TokenKind::When),
            "for"      => Some(TokenKind::For),
            "from"     => Some(TokenKind::From),
            "to"       => Some(TokenKind::To),
            "while"    => Some(TokenKind::While),
            "break"    => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "fun"      => Some(TokenKind::Fun),
            "return"   => Some(TokenKind::Return),
            "endif"    => Some(TokenKind::EndIf),
            "endfor"   => Some(TokenKind::EndFor),
            "endwhile" => Some(TokenKind::EndWhile),
            "endfun"   => Some(TokenKind::EndFun),
            "true"     => Some(TokenKind::BoolLiteral(true)),
            "false"    => Some(TokenKind::BoolLiteral(false)),
            "empty"    => Some(TokenKind::Empty),
            "read"     => Some(TokenKind::Read),
            "write"    => Some(TokenKind::Write),
            "writeln"  => Some(TokenKind::Writeln),
            _          => None,
        }
    }
}
