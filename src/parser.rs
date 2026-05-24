use std::{boxed, iter::Peekable, slice::Iter};

use anyhow::{Context, bail};

use crate::{
    expr::Expr,
    token::{Token, TokenType},
};

#[derive(Debug)]
pub struct TokenParser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> TokenParser<'a> {
    pub fn new(tokens: Peekable<Iter<'a, Token>>) -> TokenParser<'a> {
        TokenParser { tokens }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        // TODO: properly handle multi expression token lists
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(e) => {
                tracing::error!("Syntax error: {}", e);
                None
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn expression(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.equality()?;

        if self
            .tokens
            .peek()
            .is_some_and(|tok| matches!(tok.token_type, TokenType::Comma))
        {
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

            return Ok(expr);
        }

        if self
            .tokens
            .peek()
            .is_some_and(|tok| matches!(tok.token_type, TokenType::QuestionMark))
        {
            let question_mark = self.tokens.next().expect("Just peeked at this");
            let left = self.expression()?;

            if !self
                .tokens
                .next()
                .is_some_and(|tok| matches!(tok.token_type, TokenType::Colon))
            {
                bail!(
                    "Ternary operator without ':' starting on line {}",
                    question_mark.line
                )
            };

            let right = self.expression()?;
            expr = Expr::Ternary {
                condition: Box::new(expr),
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn equality(&mut self) -> anyhow::Result<Expr> {
        // equality -> comparison ( ("!=" | "==") comparison)*
        self.parse_binary_op(
            &[TokenType::EqualEqual, TokenType::BangEqual],
            TokenParser::comparison,
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn term(&mut self) -> anyhow::Result<Expr> {
        self.parse_binary_op(&[TokenType::Minus, TokenType::Plus], TokenParser::factor)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn factor(&mut self) -> anyhow::Result<Expr> {
        self.parse_binary_op(&[TokenType::Slash, TokenType::Star], TokenParser::unary)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn unary(&mut self) -> anyhow::Result<Expr> {
        tracing::debug!("unary");
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn primary(&mut self) -> anyhow::Result<Expr> {
        tracing::debug!("primary");
        if let Some(tok) = self.tokens.next() {
            tracing::debug!("token type: {:?}", tok.token_type);
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
                TokenType::Slash
                | TokenType::Star
                | TokenType::Semicolon
                | TokenType::Plus
                | TokenType::Minus
                | TokenType::Comma
                | TokenType::Bang
                | TokenType::BangEqual
                | TokenType::Equal
                | TokenType::EqualEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
                | TokenType::Less
                | TokenType::LessEqual
                | TokenType::QuestionMark
                | TokenType::Colon => {
                    let _right = self.expression();
                    tracing::error!(
                        "Got unexpected operator token: {:?} on line {}",
                        tok.token_type,
                        tok.line
                    );
                    Ok(Expr::Literal { value: None })
                }
                _ => bail!("Expected expression on line {}", tok.line),
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_binary_op(
        &mut self,
        token_types: &[TokenType],
        next_level: fn(&mut TokenParser<'a>) -> anyhow::Result<Expr>,
    ) -> anyhow::Result<Expr> {
        tracing::debug!("parsing binary op");
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
