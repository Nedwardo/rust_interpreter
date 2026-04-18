use super::expr::{Binary, Expr, Grouping, Literal, Unary};
use super::expr_visitor::ExprVisitor;

pub struct RpnPrinter;

impl ExprVisitor for RpnPrinter {
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

impl RpnPrinter {
    fn parenthesize(
        &self,
        name: &str,
        lhs: &dyn Expr,
        rhs: Option<&dyn Expr>,
    ) -> String {
        let mut output = String::with_capacity(2 * 3);

        output += &lhs.accept(self);
        if let Some(rhs_value) = rhs {
            output += " ";
            output += &rhs_value.accept(self);
        }
        output += " ";
        output += name;

        output
    }

    fn print(&self, expr: &impl Expr) -> String {
        expr.accept(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Binary, Literal, RpnPrinter};
    use crate::token::LiteralValue::Number;
    use crate::token_type::TokenType::{MINUS, PLUS, STAR};
    #[test]
    fn expression() {
        let expression = Binary {
            left: Box::new(Binary {
                operator: PLUS,
                left: Box::new(Literal { value: Number(1.0) }),
                right: Box::new(Literal { value: Number(2.0) }),
            }),
            operator: STAR,
            right: Box::new(Binary {
                operator: MINUS,
                left: Box::new(Literal { value: Number(4.0) }),
                right: Box::new(Literal { value: Number(3.0) }),
            }),
        };

        let output = RpnPrinter {}.print(&expression);
        println!("{output}");
        assert!(output == "1 2 + 4 3 - *");
    }
}
