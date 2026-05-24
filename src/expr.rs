use anyhow::bail;

use crate::token::{Token, TokenType};
use std::fmt::Display;

#[derive(Debug, Clone)]
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
        value: Object,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Number(n) => *n != 0.,
            Object::String(s) => !s.is_empty(),
            Object::Boolean(b) => *b,
            Object::Nil => false,
        }
    }

    pub fn equal(&self, right: &Object) -> bool {
        match (self, right) {
            (Object::Number(left), Object::Number(right)) => left == right,
            (Object::String(left), Object::String(right)) => left == right,
            (Object::Boolean(left), Object::Boolean(right)) => left == right,
            (Object::Nil, Object::Nil) => true,
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Object::String(value) => value.to_string(),
            Object::Number(value) => value.to_string(),
            Object::Boolean(value) => value.to_string(),
            Object::Nil => "nil".to_string(),
        };

        write!(f, "{}", str)
    }
}

pub fn evaluate(expr: Expr) -> anyhow::Result<Object> {
    Ok(match expr {
        Expr::Ternary {
            condition,
            left,
            right,
        } => {
            let condition = evaluate(*condition)?;
            if condition.is_truthy() {
                evaluate(*left)?
            } else {
                evaluate(*right)?
            }
        }
        Expr::Binary {
            left,
            right,
            operator,
        } => {
            let left = evaluate(*left)?;
            let right = evaluate(*right)?;
            if let Object::Number(left) = left
                && let Object::Number(right) = right
            {
                // handle purely double number operators
                match operator.token_type {
                    TokenType::Minus => Object::Number(left - right),
                    TokenType::Plus => Object::Number(left + right),
                    TokenType::Star => Object::Number(left * right),
                    TokenType::Slash => {
                        if right == 0. {
                            bail!("[line {}] Cannot divide by zero", operator.line)
                        } else {
                            Object::Number(left / right)
                        }
                    }
                    TokenType::Less => Object::Boolean(left < right),
                    TokenType::LessEqual => Object::Boolean(left <= right),
                    TokenType::Greater => Object::Boolean(left > right),
                    TokenType::GreaterEqual => Object::Boolean(left <= right),
                    _ => bail!(
                        "[line {}] '{}' is not a valid operator for numbers.",
                        operator.line,
                        operator.lexeme
                    ),
                }
            } else if matches!(operator.token_type, TokenType::Plus) {
                // handle plus seperately from other arithmetic operators as it can do string concat
                if let (Object::String(left), Object::String(right)) = (&left, &right) {
                    Object::String(format!("{}{}", left, right))
                } else {
                    bail!(
                        "[line {}] Operands of '+' must be two numbers or two strings, got {} and {}.",
                        operator.line,
                        left,
                        right
                    )
                }
            } else if matches!(operator.token_type, TokenType::BangEqual) {
                Object::Boolean(!left.equal(&right))
            } else if matches!(operator.token_type, TokenType::EqualEqual) {
                Object::Boolean(left.equal(&right))
            } else {
                bail!(
                    "[line {}] Operands of '{}' must be numbers, got {} and {}.",
                    operator.line,
                    operator.lexeme,
                    left,
                    right
                );
            }
        }
        Expr::Grouping { expression } => evaluate(*expression)?,
        Expr::Literal { value } => value,
        Expr::Unary { operator, right } => {
            let right = evaluate(*right)?;
            if matches!(operator.token_type, TokenType::Minus)
                && let Object::Number(n) = right
            {
                Object::Number(-n)
            } else if matches!(operator.token_type, TokenType::Bang) {
                Object::Boolean(!right.is_truthy())
            } else {
                bail!(
                    "[line {}] Operand of '{}' must be a number, got {}.",
                    operator.line,
                    operator.lexeme,
                    right
                );
            }
        }
    })
}
