use std::{iter::Peekable, slice::Iter};

use anyhow::bail;

use crate::{
    expr::Expr,
    token::{Token, TokenType},
};

pub struct TokenParser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> TokenParser<'a> {
    pub fn new(tokens: Iter<'a, Token>) -> TokenParser<'a> {
        TokenParser {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(e) => {
                tracing::error!("Syntax error: {}", e);
                None
            }
        }
    }

    fn expression(&mut self) -> anyhow::Result<Expr> {
        // we know we can just grab this as equality === expression for this purpose
        let mut expr = self.equality()?;
        while self
            .tokens
            .peek()
            .is_some_and(|tok| matches!(tok.token_type, TokenType::Comma))
        {
            let comma = self.tokens.next().expect("Just peeked at this");
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator: comma.clone(),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> anyhow::Result<Expr> {
        // equality -> comparison ( ("!=" | "==") comparison)*
        self.parse_binary_op(
            &[TokenType::EqualEqual, TokenType::BangEqual],
            TokenParser::comparison,
        )
    }

    fn comparison(&mut self) -> anyhow::Result<Expr> {
        self.parse_binary_op(
            &[
                TokenType::Less,
                TokenType::LessEqual,
                TokenType::Greater,
                TokenType::GreaterEqual,
            ],
            TokenParser::term,
        )
    }

    fn term(&mut self) -> anyhow::Result<Expr> {
        self.parse_binary_op(&[TokenType::Minus, TokenType::Plus], TokenParser::factor)
    }

    fn factor(&mut self) -> anyhow::Result<Expr> {
        self.parse_binary_op(&[TokenType::Slash, TokenType::Star], TokenParser::unary)
    }

    fn unary(&mut self) -> anyhow::Result<Expr> {
        if let Some(tok) = self.tokens.peek()
            && matches!(tok.token_type, TokenType::Bang | TokenType::Minus)
        {
            let operator = self.tokens.next().expect("Just peeked").clone();
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary { operator, right })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> anyhow::Result<Expr> {
        if let Some(tok) = self.tokens.next() {
            match &tok.token_type {
                TokenType::False => Ok(Expr::Literal {
                    value: Some(Box::new(false)),
                }),
                TokenType::True => Ok(Expr::Literal {
                    value: Some(Box::new(true)),
                }),
                TokenType::Nil => Ok(Expr::Literal { value: None }),
                TokenType::String(value) => Ok(Expr::Literal {
                    value: Some(Box::new(value.clone())),
                }),
                TokenType::Number(value) => Ok(Expr::Literal {
                    value: Some(Box::new(*value)),
                }),
                TokenType::LParen => {
                    let line = tok.line;
                    let expr = self.expression()?;
                    if !self
                        .tokens
                        .next()
                        .is_some_and(|t| matches!(t.token_type, TokenType::RParen))
                    {
                        bail!("Got invalid token list with unclosed '(' on line {}", line);
                    }
                    Ok(Expr::Grouping {
                        expression: Box::new(expr),
                    })
                }
                _ => bail!("Expected expression"),
            }
        } else {
            bail!("Unexpected end of input while parsing expression")
        }
    }

    /// Resynchronize the parser when we hit a syntax error and need to get back to a valid state
    fn synchronize(&mut self) {
        let mut last = if let Some(tok) = self.tokens.next() {
            tok
        } else {
            // if there are no more tokens we don't need to synchronize
            return;
        };

        while let Some(tok) = self.tokens.peek()
            && !matches!(tok.token_type, TokenType::Eof)
        {
            // probably just finished a statement
            if matches!(last.token_type, TokenType::Semicolon) {
                return;
            }

            // probably about to start a statement
            match last.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            last = self.tokens.next().expect("Just peeked at this");
        }
    }

    fn parse_binary_op(
        &mut self,
        token_types: &[TokenType],
        next_level: fn(&mut TokenParser<'a>) -> anyhow::Result<Expr>,
    ) -> anyhow::Result<Expr> {
        let mut expr = next_level(self)?;
        while let Some(tok) = self.tokens.peek()
            && token_types.contains(&tok.token_type)
        {
            let operator = self.tokens.next().expect("Just peeked at this").clone();
            let right = next_level(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            };
        }

        Ok(expr)
    }
}
