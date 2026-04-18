use super::expr::{Binary, Grouping, Literal, Unary};

pub trait ExprVisitor {
    fn visit_binary(&self, binary: &Binary<'_>) -> String;
    fn visit_grouping(&self, grouping: &Grouping<'_>) -> String;
    fn visit_literal(&self, literal: &Literal<'_>) -> String;
    fn visit_unary(&self, unary: &Unary<'_>) -> String;
}
