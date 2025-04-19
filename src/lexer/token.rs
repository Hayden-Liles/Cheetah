use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Keywords
    Def,
    Return,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Pass,
    Import,
    From,
    As,
    True,
    False,
    None,
    And,
    Or,
    Not,
    Class,
    With,
    Assert,
    Async,
    Await,
    Try,
    Except,
    Finally,
    Raise,
    Lambda,
    Global,
    Nonlocal,
    Yield,
    Del,
    Is,
    Match,
    Case,

    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    RawString(String),
    FString(String),
    BinaryLiteral(i64),
    OctalLiteral(i64),
    HexLiteral(i64),

    // Operators
    Plus,        // +
    Minus,       // -
    Multiply,    // *
    Divide,      // /
    FloorDivide, // //
    Modulo,      // %
    Power,       // **
    BackSlash,   // \ (for line continuations and other uses)

    Assign,           // =
    PlusAssign,       // +=
    MinusAssign,      // -=
    MulAssign,        // *=
    DivAssign,        // /=
    ModAssign,        // %=
    PowAssign,        // **=
    MatrixMulAssign,  // @=
    FloorDivAssign,   // //=
    BitwiseAndAssign, // &=
    BitwiseOrAssign,  // |=
    BitwiseXorAssign, // ^=
    ShiftLeftAssign,  // <<=
    ShiftRightAssign, // >>=

    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=

    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseNot, // ~
    ShiftLeft,  // <<
    ShiftRight, // >>

    Walrus,   // :=
    Ellipsis, // ...

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    SemiColon,    // ;
    Arrow,        // ->
    At,           // @ (for decorators)

    // Indentation (special in Python-like syntax)
    Indent,
    Dedent,
    Newline,

    // End of file
    EOF,

    // Invalid token
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub lexeme: String,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, lexeme: String) -> Self {
        Token {
            token_type,
            line,
            column,
            lexeme,
        }
    }

    pub fn error(message: &str, line: usize, column: usize, lexeme: &str) -> Self {
        Token::new(
            TokenType::Invalid(message.to_string()),
            line,
            column,
            lexeme.to_owned(),
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} '{}' at {}:{}",
            self.token_type, self.lexeme, self.line, self.column
        )
    }
}
