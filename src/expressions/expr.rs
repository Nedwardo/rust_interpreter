use super::expr_visitor::ExprVisitor;
use crate::token::LiteralValue;
use crate::token_type::TokenType;
pub trait Expr {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String;
}

pub struct Binary<'a> {
    pub left: Box<dyn Expr + 'a>,
    pub operator: TokenType,
    pub right: Box<dyn Expr + 'a>,
}

impl Expr for Binary<'_> {
    fn accept<'b>(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_binary(self)
    }
}

impl<'a> Binary<'a> {
    pub fn new(
        left: Box<dyn Expr + 'a>,
        operator: TokenType,
        right: Box<dyn Expr + 'a>,
    ) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

pub struct Grouping<'a> {
    pub expression: Box<dyn Expr + 'a>,
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
    pub operator: TokenType,
    pub expr: Box<dyn Expr + 'a>,
}

impl Expr for Unary<'_> {
    fn accept(&self, visitor: &dyn ExprVisitor) -> String {
        visitor.visit_unary(self)
    }
}
