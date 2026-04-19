pub mod printers;

use crate::token::LiteralValue;
use crate::token_type::TokenType;

pub enum Expr<'a> {
    Binary {
        left: Box<Self>,
        operator: TokenType,
        right: Box<Self>,
    },

    Grouping(Box<Self>),
    Literal(LiteralValue<'a>),
    Unary {
        operator: TokenType,
        expr: Box<Self>,
    },
}

impl Expr<'_> {
    pub fn traverse<T>(
        &self,
        collector: &impl Fn(&str, Option<&Expr>, Option<&Expr>) -> T,
    ) -> T {
        match self {
            Self::Binary {
                operator,
                left,
                right,
            } => collector(
                &operator.to_string(),
                Some(left.as_ref()),
                Some(right.as_ref()),
            ),
            Self::Grouping(expression) => {
                collector("Group", Some(expression.as_ref()), None)
            }
            Self::Literal(value) => collector(&value.to_string(), None, None),
            Self::Unary { operator, expr } => {
                collector(&operator.to_string(), Some(expr.as_ref()), None)
            }
        }
    }
}
