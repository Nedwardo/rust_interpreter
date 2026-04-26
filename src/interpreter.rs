use crate::expressions::Expr;
use crate::expressions::Expr::{Binary, Grouping, Identifier, Literal, Unary};
use crate::expressions::Value;
use crate::token_type::TokenType;

pub fn visit(expr: Expr) -> Value {
    match expr {
        Literal(value) => value,
        Identifier(name) => get_identifier(name),
        Unary { operator, expr } => visit_unary(operator, expr),
        Binary {
            left,
            operator,
            right,
        } => visit_binary(left, operator, right),
        Grouping(expr) => visit(expr),
    }
}

pub fn get_identifier(name: &str) -> Value {
    todo!()
}

pub fn visit_unary(operator: TokenType, expr: Expr) {
    let value = visit(expr);
}

pub fn visit_binary(left: Expr, operator: TokenType, right: Expr) {
    todo!()
}
