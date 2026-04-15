use super::expr::{Binary, Expr, Grouping, Literal, Unary};
use super::expr_visitor::ExprVisitor;

pub struct RpnPrinter;

impl ExprVisitor for RpnPrinter {
    fn visit_binary(&self, binary: &Binary) -> String {
        self.parenthesize(
            binary.operator.lexeme,
            binary.left,
            Some(binary.right),
        )
    }
    fn visit_grouping(&self, grouping: &Grouping) -> String {
        self.parenthesize("group", grouping.expression, None)
    }
    fn visit_literal(&self, literal: &Literal) -> String {
        // match literal.value {
        //     Some(value) => value.to_string(),
        //     None => "nil".to_owned()
        // }
        literal.value.to_string()
    }
    fn visit_unary(&self, unary: &Unary) -> String {
        self.parenthesize(unary.operator.lexeme, unary.expr, None)
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
    use crate::token::Token;
    use crate::token_type::TokenType::{MINUS, PLUS, STAR};
    #[test]
    fn expression() {
        let expression = Binary {
            operator: Token {
                token_type: STAR,
                lexeme: "*",
                literal: None,
                line: 1,
            },
            left: &Binary {
                operator: Token {
                    token_type: PLUS,
                    lexeme: "+",
                    literal: None,
                    line: 1,
                },
                left: &Literal { value: Number(1.0) },
                right: &Literal { value: Number(2.0) },
            },
            right: &Binary {
                operator: Token {
                    token_type: MINUS,
                    lexeme: "-",
                    literal: None,
                    line: 1,
                },
                left: &Literal { value: Number(4.0) },
                right: &Literal { value: Number(3.0) },
            },
        };

        let output = RpnPrinter {}.print(&expression);
        println!("{output}");
        assert!(output == "1 2 + 4 3 - *");
    }
}
