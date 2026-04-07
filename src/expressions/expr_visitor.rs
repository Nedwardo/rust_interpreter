use super::expr::{Binary, Grouping, Literal, Unary};

pub trait ExprVisitor {
    fn visit_binary(&self, binary: &Binary) -> String;
    fn visit_grouping(&self, grouping: &Grouping) -> String;
    fn visit_literal(&self, literal: &Literal) -> String;
    fn visit_unary(&self, unary: &Unary) -> String;
}
