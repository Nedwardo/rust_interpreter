use crate::token::TokenValue as TV;
use crate::token_type::TokenType;
use std::fmt;
use std::fmt::{Display, Formatter, Write};

pub enum Expr<'a> {
    Literal(Value),
    Identifier(&'a str),
    Unary {
        operator: UnaryOperator,
        expr: Box<Self>,
    },
    Grouping(Box<Self>),
    Binary {
        left: Box<Self>,
        operator: TokenType,
        right: Box<Self>,
    },
}

pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[allow(
    non_camel_case_types,
    clippy::upper_case_acronyms,
    reason = "Using the same names as from the book"
)]
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum UnaryOperator {
    MINUS,
    BANG,
}

impl TryFrom<TokenType> for UnaryOperator {
    type Error = ();

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::MINUS => Ok(Self::MINUS),
            TokenType::BANG => Ok(Self::BANG),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<TV<'a>> for Expr<'a> {
    type Error = ();

    fn try_from(token_value: TV<'a>) -> Result<Self, Self::Error> {
        let expr = match token_value {
            TV::String(text) => Expr::Literal(Value::String(text.to_owned())),
            TV::Number(number) => Expr::Literal(Value::Number(number)),
            TV::False => Expr::Literal(Value::Boolean(false)),
            TV::True => Expr::Literal(Value::Boolean(true)),
            TV::Nil => Expr::Literal(Value::Nil),
            TV::Identifier(name) => Expr::Identifier(name),
            TV::Comment(..) => return Err(()),
        };
        Ok(expr)
    }
}

impl Expr<'_> {
    pub fn visit<T>(
        &self,
        fold: &impl Fn(&str, Option<&Expr>, Option<&Expr>) -> T,
    ) -> T {
        match self {
            Self::Binary {
                operator,
                left,
                right,
            } => fold(
                &operator.to_string(),
                Some(left.as_ref()),
                Some(right.as_ref()),
            ),
            Self::Grouping(expression) => {
                fold("Group", Some(expression.as_ref()), None)
            }
            Self::Literal(value) => fold(&value.to_string(), None, None),
            Self::Unary { operator, expr } => {
                fold(&operator.to_string(), Some(expr.as_ref()), None)
            }
            Self::Identifier(name) => fold(name, None, None),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MINUS => write!(f, "-"),
            Self::BANG => write!(f, "!"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "\"{value}\""),
            Self::Number(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Nil => write!(f, "Nil"),
        }
    }
}

pub fn format_ast(expr: &Expr) -> String {
    expr.visit(&ast_print_node)
}

pub fn format_rpn(expr: &Expr) -> String {
    expr.visit(&rpn_print_node)
}

#[allow(unused, reason = "string write! cannot fail")]
fn ast_print_node(
    name: &str,
    lhs: Option<&Expr>,
    rhs: Option<&Expr>,
) -> String {
    lhs.map_or_else(
        || name.to_owned(),
        |left_child| {
            let mut output = String::with_capacity(2 * 3);

            write!(
                &mut output,
                "({} {}",
                name,
                left_child.visit(&ast_print_node)
            );
            if let Some(right_child) = rhs {
                write!(&mut output, " {}", right_child.visit(&ast_print_node));
            }
            write!(&mut output, ")");

            output
        },
    )
}

#[allow(unused, reason = "string write! cannot fail")]
fn rpn_print_node(
    name: &str,
    lhs: Option<&Expr>,
    rhs: Option<&Expr>,
) -> String {
    let mut output = String::with_capacity(2 * 3);

    if let Some(left_child) = lhs {
        write!(&mut output, "{} ", left_child.visit(&rpn_print_node));
        if let Some(right_child) = rhs {
            write!(&mut output, "{} ", right_child.visit(&rpn_print_node));
        }
    }
    write!(&mut output, "{name}");

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::Expr::{Binary, Grouping, Literal, Unary};
    use crate::token_type::TokenType::{MINUS, PLUS, STAR};
    #[test]
    fn ast() {
        let expression = Binary {
            left: Box::new(Unary {
                operator: UnaryOperator::MINUS,
                expr: Box::new(Literal(Value::Number(123.0))),
            }),
            operator: STAR,
            right: Box::new(Grouping(Box::new(Literal(Value::Number(45.67))))),
        };

        let output = format_ast(&expression);
        println!("{output}");
        assert!(output == "(* (- 123) (Group 45.67))");
    }

    #[test]
    fn rpn() {
        let expression = Binary {
            left: Box::new(Binary {
                operator: PLUS,
                left: Box::new(Literal(Value::Number(1.0))),
                right: Box::new(Literal(Value::Number(2.0))),
            }),
            operator: STAR,
            right: Box::new(Binary {
                operator: MINUS,
                left: Box::new(Literal(Value::Number(4.0))),
                right: Box::new(Literal(Value::Number(3.0))),
            }),
        };

        assert!(format_rpn(&expression) == "1 2 + 4 3 - *");
    }
}
