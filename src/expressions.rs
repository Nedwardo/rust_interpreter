use crate::token_type::TokenType;
use crate::token_type::operator_subset;
use std::fmt;
use std::fmt::{Display, Formatter};

pub enum Statment<'a> {
    Declaration {
        name: &'a str,
        expression: Option<Expr<'a>>,
    },
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Group(Vec<Self>),
}

#[derive(Debug)]
pub struct Expr<'a> {
    pub line: usize,
    pub kind: ExprKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
    Literal(Value),
    Identifier(&'a str),
    Unary(Unary<'a>),
    Grouping(Box<Expr<'a>>),
    Binary(Binary<'a>),
    Assignment(Assignment<'a>),
}

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Debug)]
pub struct Assignment<'a> {
    pub name: &'a str,
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Unary<'a> {
    pub operator: UnaryOperator,
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: BinaryOperator,
    pub right: Box<Expr<'a>>,
}

operator_subset!(UnaryOperator, {MINUS, BANG});
operator_subset!(BinaryOperator, {
    MINUS,
    PLUS,
    GREATER,
    GREATER_EQUAL,
    BANG_EQUAL,
    EQUAL_EQUAL,
    SLASH,
    STAR,
    COMMA,
    QUESTION_MARK,
    COLON,
    LESS,
    LESS_EQUAL,
});

impl<'a> Expr<'a> {
    pub const fn new_binary(
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Binary(Binary {
                left,
                operator,
                right,
            }),
        }
    }

    pub const fn new_unary(
        operator: UnaryOperator,
        expr: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Unary(Unary { operator, expr }),
        }
    }

    pub const fn new_literal(value: Value, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Literal(value),
        }
    }

    pub const fn new_identifier(identifier: &'a str, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Identifier(identifier),
        }
    }

    pub const fn new_grouping(grouping: Box<Self>, line: usize) -> Self {
        Expr {
            line,
            kind: ExprKind::Grouping(grouping),
        }
    }

    pub const fn new_assignment(
        name: &'a str,
        expr: Box<Self>,
        line: usize,
    ) -> Self {
        Expr {
            line,
            kind: ExprKind::Assignment(Assignment { name, expr }),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::String(..) => write!(f, "\"{}\"", self.cast_to_string()),
            _ => write!(f, "{}", self.cast_to_string()),
        }
    }
}

impl Value {
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(..) => "String",
            Self::Number(..) => "Number",
            Self::Boolean(..) => "Boolean",
            Self::Nil => "nil",
        }
    }

    pub fn cast_to_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Nil => "nil".to_owned(),
        }
    }
}
