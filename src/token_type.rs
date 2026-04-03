use std::fmt::{Display, Formatter, Result};

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

impl Display for TokenType {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        let output = match self {
            Self::LEFT_PAREN => "(",
            Self::RIGHT_PAREN => ")",
            Self::LEFT_BRACE => "{",
            Self::RIGHT_BRACE => "}",
            Self::COMMA => "(",
            Self::DOT => ")",
            Self::MINUS => "{",
            Self::PLUS => "}",
            Self::SEMICOLON => "(",
            Self::SLASH => ")",
            Self::STAR => "{",

            Self::BANG => "!",
            Self::BANG_EQUAL => "!=",
            Self::EQUAL => "=",
            Self::EQUAL_EQUAL => "==",
            Self::GREATER => ">",
            Self::GREATER_EQUAL => ">=",
            Self::LESS => "<",
            Self::LESS_EQUAL => "<=",

            Self::IDENTIFIER => "case",
            Self::STRING => "\"\"",
            Self::NUMBER => "1234....",

            Self::AND => "and",
            Self::CLASS => "class",
            Self::ELSE => "else",
            Self::FALSE => "false",
            Self::FUN => "fun",
            Self::FOR => "for",
            Self::IF => "if",
            Self::NIL => "NIL",
            Self::OR => "or",
            Self::PRINT => "print",
            Self::RETURN => "return",
            Self::SUPER => "super",
            Self::THIS => "this",
            Self::TRUE => "true",
            Self::VAR => "var",
            Self::WHILE => "while",

            Self::EOF => "\\0",
        };
        write!(formatter, "{}", output)
    }
}
