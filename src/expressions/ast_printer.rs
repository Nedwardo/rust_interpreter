use super::expr::{Binary, Expr, Grouping, Literal, Unary};
use super::expr_visitor::ExprVisitor;

pub struct AstPrinter;

impl ExprVisitor for AstPrinter {
    fn visit_binary(&self, binary: &Binary<'_>) -> String {
        self.parenthesize(
            &binary.operator.to_string(),
            binary.left.as_ref(),
            Some(binary.right.as_ref()),
        )
    }
    fn visit_grouping(&self, grouping: &Grouping<'_>) -> String {
        self.parenthesize("group", grouping.expression.as_ref(), None)
    }
    fn visit_literal(&self, literal: &Literal) -> String {
        literal.value.to_string()
    }
    fn visit_unary(&self, unary: &Unary<'_>) -> String {
        self.parenthesize(
            &unary.operator.to_string(),
            unary.expr.as_ref(),
            None,
        )
    }
}

impl AstPrinter {
    fn parenthesize(
        &self,
        name: &str,
        lhs: &dyn Expr,
        rhs: Option<&dyn Expr>,
    ) -> String {
        let mut output = String::with_capacity(2 * 3);

        output += "(";
        output += name;
        output += " ";
        output += &lhs.accept(self);
        if let Some(rhs_value) = rhs {
            output += " ";
            output += &rhs_value.accept(self);
        }
        output += ")";

        output
    }

    fn print(&self, expr: &impl Expr) -> String {
        expr.accept(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{AstPrinter, Binary, Grouping, Literal, Unary};
    use crate::token::LiteralValue::Number;
    use crate::token_type::TokenType::{MINUS, STAR};
    #[test]
    fn expression() {
        let expression = Binary {
            left: Box::new(Unary {
                operator: MINUS,
                expr: Box::new(Literal {
                    value: Number(123.0),
                }),
            }),
            operator: STAR,
            right: Box::new(Grouping {
                expression: Box::new(Literal {
                    value: Number(45.67),
                }),
            }),
        };

        assert!(
            AstPrinter {}.print(&expression) == "(* (- 123) (group 45.67))"
        );
    }
}
