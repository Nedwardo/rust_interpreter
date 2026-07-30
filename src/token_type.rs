use core::fmt::{Debug, Display, Formatter};

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
    QUESTION_MARK,
    COLON,

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
    BREAK,

    COMMENT,
}

operator_subset!(ValueTokenTypes, {STRING, NUMBER, TRUE, FALSE, NIL, IDENTIFIER, COMMENT});

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
            "break" => Some(BREAK),
            _ => None,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
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
            QUESTION_MARK => "?",
            COLON => ":",

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
            BREAK => "break",
        };
        write!(f, "{output}")
    }
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

pub trait OperatorSubset<P: 'static>:
    TryFrom<P> + PartialEq + Eq + Copy + Clone
{
    const VARIANTS: &'static [P];
}

macro_rules! operator_subset {
    ($name:ident, { $($variant:ident),* $(,)? }) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        pub enum $name { $($variant),* }

        impl OperatorSubset<crate::token_type::TokenType> for $name {
            const VARIANTS: &'static [crate::token_type::TokenType] = &[$(crate::token_type::TokenType::$variant),*];
        }

        impl std::convert::TryFrom<crate::token_type::TokenType> for $name {
            type Error = ();
            fn try_from(tt: crate::token_type::TokenType) -> std::result::Result<Self, ()> {
                match tt {
                    $(crate::token_type::TokenType::$variant => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }

         impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self { $(Self::$variant => std::fmt::Display::fmt(&crate::token_type::TokenType::$variant, f)),* }
            }
        }
    };
}
pub(crate) use operator_subset;
