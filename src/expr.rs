use crate::token::Token;
use std::any::Any;

#[derive(Debug)]
pub enum Expr {
    Ternary {
        condition: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        /// Placeholder for java `Object` keyword which is the base class that everything inherits from
        value: Option<Box<dyn Any>>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}
