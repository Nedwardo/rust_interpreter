use crate::evaluator::environment::{Frame, VariableBinding};
use crate::operator_subset;
use crate::token::{Span, Token};
use crate::token_type::KeywordToken as KT;
use crate::token_type::LiteralToken as LT;
use crate::token_type::TokenType as TT;
use crate::token_type::TokenTypeSubset;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum Statement<'a> {
    Declaration {
        binding: VariableBinding<'a>,
        expression: Option<Expr<'a>>,
    },
    FunctionDeclaration(Function<'a>),
    Expression(Expr<'a>),
    Print(Expr<'a>),
    Group(Vec<Self>),
    If {
        condition: Expr<'a>,
        true_branch: Box<Self>,
        false_branch: Option<Box<Self>>,
    },
    While {
        condition: Expr<'a>,
        body: Box<Self>,
    },
    Return {
        span: Span,
        value: Option<Expr<'a>>,
    },
    Break,
}

impl<'a> Statement<'a> {
    #[must_use]
    pub fn r#for(
        initialiser: Option<Self>,
        condition: Option<Expr<'a>>,
        increment: Option<Expr<'a>>,
        body: Box<Self>,
        span: Span,
    ) -> Self {
        let body = match increment {
            Some(inc) => {
                Box::new(Self::Group(vec![*body, Self::Expression(inc)]))
            }
            None => body,
        };
        let flattened_condition = condition
            .unwrap_or_else(|| Expr::literal(Value::Boolean(true), span));

        let while_loop = Self::While {
            condition: flattened_condition,
            body,
        };

        match initialiser {
            None => while_loop,
            Some(init) => Self::Group(vec![init, while_loop]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr<'a> {
    pub span: Span,
    pub kind: ExprKind<'a>,
}

#[derive(Clone, Debug)]
pub enum ExprKind<'a> {
    Literal(Value<'a>),
    Identifier(&'a str),
    Unary(Unary<'a>),
    Grouping(Box<Expr<'a>>),
    Binary(Binary<'a>),
    Assignment(Assignment<'a>),
    Logical(Logical<'a>),
    Call(Call<'a>),
    Lambda(Function<'a>),
}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Function {
        declaration: Function<'a>,
        closure: Frame<'a>,
    },
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Clone, Debug)]
pub enum ValueError {
    NotANumber(Span),
    NotAValue(Span),
}

#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    pub name: &'a str,
    pub expr: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Unary<'a> {
    pub operator: UnaryOperator,
    pub expr: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: BinaryOperator,
    pub right: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Logical<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: LogicalOperator,
    pub right: Box<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Call<'a> {
    pub callee: Box<Expr<'a>>,
    pub arguments: Vec<Expr<'a>>,
}

#[derive(Clone, Debug)]
pub struct Function<'a> {
    pub binding: VariableBinding<'a>,
    pub body: FunctionKind<'a>,
    pub params: Vec<VariableBinding<'a>>,
}

#[derive(Clone, Debug)]
pub enum FunctionKind<'a> {
    Lox(Box<Statement<'a>>),
    Rust(fn(Vec<Value<'a>>) -> Value<'a>),
}

operator_subset!(UnaryOperator, {Minus, Bang});
operator_subset!(BinaryOperator, {
    Minus,
    Plus,
    Greater,
    GreaterEqual,
    BangEqual,
    EqualEqual,
    Slash,
    Star,
    Comma,
    QuestionMark,
    Colon,
    Less,
    LessEqual,
});
operator_subset!(LogicalOperator, {Or, And});

impl<'a> Expr<'a> {
    #[must_use]
    pub const fn logical(
        left: Box<Self>,
        operator: LogicalOperator,
        right: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Logical(Logical {
                left,
                operator,
                right,
            }),
        }
    }

    #[must_use]
    pub const fn binary(
        left: Box<Self>,
        operator: BinaryOperator,
        right: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Binary(Binary {
                left,
                operator,
                right,
            }),
        }
    }

    #[must_use]
    pub const fn unary(
        operator: UnaryOperator,
        expr: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Unary(Unary { operator, expr }),
        }
    }

    #[must_use]
    pub const fn literal(value: Value<'a>, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Literal(value),
        }
    }

    #[must_use]
    pub const fn identifier(identifier: &'a str, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Identifier(identifier),
        }
    }

    #[must_use]
    pub const fn grouping(grouping: Box<Self>, span: Span) -> Self {
        Expr {
            span,
            kind: ExprKind::Grouping(grouping),
        }
    }

    #[must_use]
    pub const fn assignment(
        name: &'a str,
        expr: Box<Self>,
        span: Span,
    ) -> Self {
        Expr {
            span,
            kind: ExprKind::Assignment(Assignment { name, expr }),
        }
    }

    #[must_use]
    pub const fn call(callee: Box<Self>, arguments: Vec<Self>) -> Self {
        Expr {
            span: callee.span,
            kind: ExprKind::Call(Call { callee, arguments }),
        }
    }

    #[must_use]
    pub const fn lambda(function: Function<'a>) -> Self {
        Expr {
            span: function.binding.span,
            kind: ExprKind::Lambda(function),
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Function {
                declaration:
                    Function {
                        binding: VariableBinding { name, .. },
                        ..
                    },
                ..
            } => write!(f, "<fn {name}>"),
            _ => write!(f, "{}", self.cast_to_string()),
        }
    }
}

impl<'a> Value<'a> {
    #[must_use]
    pub const fn token_types() -> &'static [TT] {
        &[
            TT::Literal(LT::String),
            TT::Keyword(KT::True),
            TT::Keyword(KT::False),
            TT::Literal(LT::Number),
            TT::Keyword(KT::Nil),
        ]
    }

    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::String(..) => "String",
            Self::Number(..) => "Number",
            Self::Boolean(..) => "Boolean",
            Self::Nil => "nil",
            Self::Function { .. } => "Function",
        }
    }

    #[must_use]
    pub fn cast_to_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => format!("{value}"),
            Self::Boolean(value) => format!("{value}"),
            Self::Nil => "nil".to_owned(),
            Self::Function { .. } => "Function".to_owned(),
        }
    }

    /// # Errors
    ///
    /// Will error if the number is invalid
    pub fn try_from(
        token: &Token,
        source: &'a str,
    ) -> Result<Self, ValueError> {
        let sub_string = &source[token.span.0..token.span.1];
        match token.token_kind {
            TT::Literal(LT::String) => {
                Ok(Self::String(sub_string[1..sub_string.len() - 1].to_owned()))
            }
            TT::Keyword(KT::True) => Ok(Self::Boolean(true)),
            TT::Keyword(KT::False) => Ok(Self::Boolean(false)),
            TT::Literal(LT::Number) => sub_string
                .parse::<f64>()
                .map_or(Err(ValueError::NotANumber(token.span)), |v| {
                    Ok(Self::Number(v))
                }),
            TT::Keyword(KT::Nil) => Ok(Self::Nil),
            _ => Err(ValueError::NotAValue(token.span)),
        }
    }
}
