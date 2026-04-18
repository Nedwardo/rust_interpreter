use core::fmt::{Debug, Display, Formatter, Result};

#[allow(
    non_camel_case_types,
    clippy::upper_case_acronyms,
    reason = "Using the same names as from the book"
)]
#[derive(PartialEq, Eq, Copy, Clone)]
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

    COMMENT,
    EOF,
}

#[allow(clippy::enum_glob_use, reason = "Too many enum groups")]
use super::token_type::TokenType::*;
impl TokenType {
    pub fn from_lexeme(keyword: &str) -> Option<Self> {
        match keyword {
            "and" => Some(AND),
            "class" => Some(CLASS),
            "else" => Some(ELSE),
            "false" => Some(FALSE),
            "for" => Some(FOR),
            "fun" => Some(FUN),
            "if" => Some(IF),
            "nil" => Some(NIL),
            "or" => Some(OR),
            "print" => Some(PRINT),
            "return" => Some(RETURN),
            "super" => Some(SUPER),
            "this" => Some(THIS),
            "true" => Some(TRUE),
            "var" => Some(VAR),
            "while" => Some(WHILE),
            _ => None,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let output = match *self {
            LEFT_PAREN => "(",
            RIGHT_PAREN => ")",
            LEFT_BRACE => "{",
            RIGHT_BRACE => "}",
            COMMA => ",",
            DOT => ".",
            MINUS => "-",
            PLUS => "+",
            SEMICOLON => ";",
            SLASH => "/",
            STAR => "*",

            BANG => "!",
            BANG_EQUAL => "!=",
            EQUAL => "=",
            EQUAL_EQUAL => "==",
            GREATER => ">",
            GREATER_EQUAL => ">=",
            LESS => "<",
            LESS_EQUAL => "<=",

            IDENTIFIER => "{IDENTIFIER}",
            STRING => "{STRING}",
            NUMBER => "{NUMBER}",
            COMMENT => "{COMMENT}",

            AND => "and",
            CLASS => "class",
            ELSE => "else",
            FALSE => "false",
            FUN => "fun",
            FOR => "for",
            IF => "if",
            NIL => "NIL",
            OR => "or",
            PRINT => "print",
            RETURN => "return",
            SUPER => "super",
            THIS => "this",
            TRUE => "true",
            VAR => "var",
            WHILE => "while",

            EOF => "EOF",
        };
        write!(f, "{output}")
    }
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(self, f)
    }
}
