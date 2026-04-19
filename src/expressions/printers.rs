use super::Expr;
use std::fmt::Write;

pub fn ast_print(expr: &Expr) -> String {
    expr.traverse(&ast_print_node)
}

pub fn rpn_print(expr: &Expr) -> String {
    expr.traverse(&rpn_print_node)
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
                left_child.traverse(&ast_print_node)
            );
            if let Some(right_child) = rhs {
                write!(
                    &mut output,
                    " {}",
                    right_child.traverse(&ast_print_node)
                );
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
        write!(&mut output, "{} ", left_child.traverse(&rpn_print_node));
        if let Some(right_child) = rhs {
            write!(&mut output, "{} ", right_child.traverse(&rpn_print_node));
        }
    }
    write!(&mut output, "{name}");

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::Expr::{Binary, Grouping, Literal, Unary};
    use crate::token::LiteralValue::Number;
    use crate::token_type::TokenType::{MINUS, PLUS, STAR};
    #[test]
    fn ast() {
        let expression = Binary {
            left: Box::new(Unary {
                operator: MINUS,
                expr: Box::new(Literal(Number(123.0))),
            }),
            operator: STAR,
            right: Box::new(Grouping(Box::new(Literal(Number(45.67))))),
        };

        let output = ast_print(&expression);
        println!("{output}");
        assert!(output == "(* (- 123) (Group 45.67))");
    }

    #[test]
    fn rpn() {
        let expression = Binary {
            left: Box::new(Binary {
                operator: PLUS,
                left: Box::new(Literal(Number(1.0))),
                right: Box::new(Literal(Number(2.0))),
            }),
            operator: STAR,
            right: Box::new(Binary {
                operator: MINUS,
                left: Box::new(Literal(Number(4.0))),
                right: Box::new(Literal(Number(3.0))),
            }),
        };

        assert!(rpn_print(&expression) == "1 2 + 4 3 - *");
    }
}
