use super::expr_visitor::ExprVisitor;
use crate::token::LiteralValue;
use crate::token::Token;
pub trait Expr {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String;
}

pub struct Binary<'a> {
    pub left: &'a dyn Expr,
    pub operator: Token<'a>,
    pub right: &'a dyn Expr,
}

impl Expr for Binary<'_> {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_binary(self)
    }
}

pub struct Grouping<'a> {
    pub expression: &'a dyn Expr,
}

impl Expr for Grouping<'_> {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_grouping(self)
    }
}

pub struct Literal<'a> {
    pub value: LiteralValue<'a>,
}

impl Expr for Literal<'_> {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_literal(self)
    }
}

pub struct Unary<'a> {
    pub operator: Token<'a>,
    pub expr: &'a dyn Expr,
}

impl Expr for Unary<'_> {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_unary(self)
    }
}
