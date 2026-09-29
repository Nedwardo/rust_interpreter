use core::fmt::{Debug, Display, Formatter};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TokenType {
    Literal(LiteralToken),
    Keyword(KeywordToken),
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum LiteralToken {
    String,
    Number,
    Identifier,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum KeywordToken {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    QuestionMark,
    Colon,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,
}

impl TokenType {
    pub fn from_lexeme(keyword: &str) -> Option<Self> {
        match keyword {
            "and" => Some(Self::Keyword(KeywordToken::And)),
            "class" => Some(Self::Keyword(KeywordToken::Class)),
            "else" => Some(Self::Keyword(KeywordToken::Else)),
            "false" => Some(Self::Keyword(KeywordToken::False)),
            "for" => Some(Self::Keyword(KeywordToken::For)),
            "fun" => Some(Self::Keyword(KeywordToken::Fun)),
            "if" => Some(Self::Keyword(KeywordToken::If)),
            "nil" => Some(Self::Keyword(KeywordToken::Nil)),
            "or" => Some(Self::Keyword(KeywordToken::Or)),
            "print" => Some(Self::Keyword(KeywordToken::Print)),
            "return" => Some(Self::Keyword(KeywordToken::Return)),
            "super" => Some(Self::Keyword(KeywordToken::Super)),
            "this" => Some(Self::Keyword(KeywordToken::This)),
            "true" => Some(Self::Keyword(KeywordToken::True)),
            "var" => Some(Self::Keyword(KeywordToken::Var)),
            "while" => Some(Self::Keyword(KeywordToken::While)),
            "break" => Some(Self::Keyword(KeywordToken::Break)),
            _ => None,
        }
    }
}

impl PartialEq<KeywordToken> for TokenType {
    fn eq(&self, other: &KeywordToken) -> bool {
        if let Self::Keyword(kt) = self
            && kt == other
        {
            true
        } else {
            false
        }
    }
}

impl PartialEq<LiteralToken> for TokenType {
    fn eq(&self, other: &LiteralToken) -> bool {
        if let Self::Literal(lt) = self
            && lt == other
        {
            true
        } else {
            false
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Keyword(keyword) => std::fmt::Display::fmt(keyword, f),
            Self::Literal(literal) => std::fmt::Display::fmt(literal, f),
        }
    }
}

impl Display for KeywordToken {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let output = match *self {
            Self::LeftParen => "(",
            Self::RightParen => ")",
            Self::LeftBrace => "{",
            Self::RightBrace => "}",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Minus => "-",
            Self::Plus => "+",
            Self::Semicolon => ";",
            Self::Slash => "/",
            Self::Star => "*",
            Self::QuestionMark => "?",

            Self::Colon => ":",
            Self::Bang => "!",
            Self::BangEqual => "!=",
            Self::Equal => "=",
            Self::EqualEqual => "==",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",

            Self::And => "and",
            Self::Class => "class",
            Self::Else => "else",
            Self::False => "false",
            Self::Fun => "fun",
            Self::For => "for",
            Self::If => "if",
            Self::Nil => "nil",
            Self::Or => "or",
            Self::Print => "print",
            Self::Return => "return",
            Self::Super => "super",
            Self::This => "this",
            Self::True => "true",
            Self::Var => "var",
            Self::While => "while",
            Self::Break => "break",
        };
        write!(f, "{output}")
    }
}

impl Display for LiteralToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Identifier => "{identifier}",
            Self::String => "{string}",
            Self::Number => "{number}",
        };
        write!(f, "{output}")
    }
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

pub trait TokenTypeSubset:
    TryFrom<TokenType> + TryFrom<KeywordToken> + PartialEq + Eq + Copy + Clone
{
    const VARIANTS: &'static [TokenType];
}

#[macro_export]
macro_rules! operator_subset {
    ($name:ident, { $($variant:ident),* $(,)? }) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        pub enum $name { $($variant),* }

        impl TokenTypeSubset for $name {
            const VARIANTS: &'static [$crate::token_type::TokenType] = &[$($crate::token_type::TokenType::Keyword($crate::token_type::KeywordToken::$variant)),*];
        }

        impl std::convert::TryFrom<$crate::token_type::TokenType> for $name {
            type Error = ();
            fn try_from(tt: $crate::token_type::TokenType) -> std::result::Result<Self, ()> {
                match tt {
                    $($crate::token_type::TokenType::Keyword($crate::token_type::KeywordToken::$variant) => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }

        impl std::convert::TryFrom<$crate::token_type::KeywordToken> for $name {
            type Error = ();
            fn try_from(tt: $crate::token_type::KeywordToken) -> std::result::Result<Self, ()> {
                match tt {
                    $($crate::token_type::KeywordToken::$variant => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }

         impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self { $(Self::$variant => std::fmt::Display::fmt(&$crate::token_type::KeywordToken::$variant, f)),* }
            }
        }
    };
}
