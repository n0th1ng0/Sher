#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Func,
    Let,
    Var,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Import,
    Struct,
    Enum,
    Print,
    True,
    False,
    Null,
    Return,

    // Types
    TypeInt8,
    TypeInt16,
    TypeInt26,
    TypeInt32,
    TypeInt64,
    TypeFloat8,
    TypeFloat16,
    TypeFloat32,
    TypeFloat64,
    TypeChar,
    TypeString,
    TypeBool,
    TypeVoid,
    TypeAny,

    // Identifiers & Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),

    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Assign,         // =
    Equal,          // ==
    NotEqual,       // !=
    Greater,        // >
    GreaterEq,      // >=
    Less,           // <
    LessEq,         // <=
    And,            // &&
    Or,             // ||
    Not,            // !

    // Increment / Decrement / Compound
    PlusPlus,       // ++
    MinusMinus,     // --
    PlusAssign,     // +=
    MinusAssign,    // -=
    StarAssign,     // *=
    SlashAssign,    // /=
    PercentAssign,  // %=

    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Colon,          // :
    ColonColon,     // ::
    Semicolon,      // ;
    Dot,            // .
    DotDot,         // ..

    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(token_type: TokenType, span: Span, lexeme: impl Into<String>) -> Self {
        Self {
            token_type,
            span,
            lexeme: lexeme.into(),
        }
    }
}
