use crate::token_type::TokenType;
use crate::token_type::operator_subset;
use std::fmt;
use std::fmt::{Display, Formatter, Write};

pub struct Expr<'a> {
    pub line: usize,
    pub kind: ExprKind<'a>,
}

pub enum ExprKind<'a> {
    Literal(Value),
    Identifier(&'a str),
    Unary(Unary<'a>),
    Grouping(Box<Expr<'a>>),
    Binary(Binary<'a>),
}

#[derive(Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

pub struct Unary<'a> {
    pub operator: UnaryOperator,
    pub expr: Box<Expr<'a>>,
}

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

    pub fn visit<T>(
        &self,
        fold: &impl Fn(&str, Option<&Expr>, Option<&Expr>) -> T,
    ) -> T {
        match &self.kind {
            ExprKind::Binary(Binary {
                operator,
                left,
                right,
            }) => fold(
                &operator.to_string(),
                Some(left.as_ref()),
                Some(right.as_ref()),
            ),
            ExprKind::Grouping(expression) => {
                fold("Group", Some(expression.as_ref()), None)
            }
            ExprKind::Literal(value) => fold(&value.to_string(), None, None),
            ExprKind::Unary(Unary { operator, expr }) => {
                fold(&operator.to_string(), Some(expr.as_ref()), None)
            }
            ExprKind::Identifier(name) => fold(name, None, None),
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
    use super::BinaryOperator as BO;
    use super::UnaryOperator as UO;
    use super::*;
    #[test]
    fn ast() {
        let expression = Expr::new_binary(
            Box::new(Expr::new_unary(
                UO::MINUS,
                Box::new(Expr::new_literal(Value::Number(123.0), 0)),
                0,
            )),
            BO::STAR,
            Box::new(Expr::new_grouping(
                Box::new(Expr::new_literal(Value::Number(45.67), 0)),
                0,
            )),
            0,
        );

        let output = format_ast(&expression);
        println!("{output}");
        assert!(output == "(* (- 123) (Group 45.67))");
    }

    #[test]
    fn rpn() {
        let expression = Expr::new_binary(
            Box::new(Expr::new_binary(
                Box::new(Expr::new_literal(Value::Number(1.0), 0)),
                BO::PLUS,
                Box::new(Expr::new_literal(Value::Number(2.0), 0)),
                0,
            )),
            BO::STAR,
            Box::new(Expr::new_binary(
                Box::new(Expr::new_literal(Value::Number(4.0), 0)),
                BO::MINUS,
                Box::new(Expr::new_literal(Value::Number(3.0), 0)),
                0,
            )),
            0,
        );

        assert!(format_rpn(&expression) == "1 2 + 4 3 - *");
    }
}
