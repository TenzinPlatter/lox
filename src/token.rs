use std::{iter::Peekable, str::Chars, vec};

use anyhow::bail;

#[derive(Debug)]
pub enum TokenType {
    /// One char tokens
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    /// One or two char tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    /// Literals
    Identifier(String),
    String(String),
    Number(f64),

    /// Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl TokenType {
    /// Checks if the passed `value` is a reserved keyword and returns the associated `TokenType` if
    /// it is
    fn from_keyword(value: &str) -> Option<TokenType> {
        match value {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: u64,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: u64) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

#[derive(Default)]
pub struct TokenParser {
    block_comment_level: u8,
}

impl TokenParser {
    pub fn parse_tokens(&mut self, source: &str) -> anyhow::Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut line: u64 = 0;
        let mut had_error = false;

        let mut chars = source.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                // we do this check first so we can exit block comments
                '*' => {
                    if let Some(next) = chars.peek()
                        && *next == '/'
                    {
                        if self.block_comment_level == 0 {
                            tracing::error!("Got closing block comment without opening '/*'");
                            had_error = true;
                        }
                        self.block_comment_level -= 1;
                        let _ = chars.next();
                        continue;
                    }
                    tokens.push(Token::new(TokenType::Star, "*".into(), line))
                }
                // this check will skip any char while we are in a block comment
                _ if self.block_comment_level > 0 => (),

                ' ' | '\r' | '\t' => (),
                '\n' => line += 1,
                '(' => tokens.push(Token::new(TokenType::RParen, "(".into(), line)),
                ')' => tokens.push(Token::new(TokenType::LParen, ")".into(), line)),
                '{' => tokens.push(Token::new(TokenType::RBrace, "{".into(), line)),
                '}' => tokens.push(Token::new(TokenType::LBrace, "}".into(), line)),
                ',' => tokens.push(Token::new(TokenType::Comma, ",".into(), line)),
                '.' => tokens.push(Token::new(TokenType::Dot, ".".into(), line)),
                '-' => tokens.push(Token::new(TokenType::Minus, "-".into(), line)),
                '+' => tokens.push(Token::new(TokenType::Plus, "+".into(), line)),
                ';' => tokens.push(Token::new(TokenType::Semicolon, ";".into(), line)),
                '!' => {
                    let (token_type, lexeme) = if let Some(next) = chars.peek()
                        && *next == '='
                    {
                        chars.next();
                        (TokenType::BangEqual, "!=".to_string())
                    } else {
                        (TokenType::Bang, "!".to_string())
                    };
                    tokens.push(Token::new(token_type, lexeme, line));
                }
                '=' => {
                    let (token_type, lexeme) = if let Some(next) = chars.peek()
                        && *next == '='
                    {
                        chars.next();
                        (TokenType::EqualEqual, "==".to_string())
                    } else {
                        (TokenType::Equal, "=".to_string())
                    };
                    tokens.push(Token::new(token_type, lexeme, line));
                }
                '<' => {
                    let (token_type, lexeme) = if let Some(next) = chars.peek()
                        && *next == '='
                    {
                        chars.next();
                        (TokenType::LessEqual, "<=".to_string())
                    } else {
                        (TokenType::Less, "<".to_string())
                    };
                    tokens.push(Token::new(token_type, lexeme, line));
                }
                '>' => {
                    let (token_type, lexeme) = if let Some(next) = chars.peek()
                        && *next == '='
                    {
                        chars.next();
                        (TokenType::GreaterEqual, ">=".to_string())
                    } else {
                        (TokenType::Greater, ">".to_string())
                    };
                    tokens.push(Token::new(token_type, lexeme, line));
                }
                '/' => {
                    if let Some(next) = chars.peek() {
                        if *next == '/' {
                            // inline comment, we need to skip the rest of the line
                            while let Some(c) = chars.next()
                                && c != '\n'
                            {}
                            continue;
                        } else if *next == '*' {
                            // block comment
                            self.block_comment_level += 1;
                            let _ = chars.next();
                            continue;
                        }
                    }
                    tokens.push(Token::new(TokenType::Slash, "/".into(), line));
                }
                '"' => {
                    let mut str_chars: Vec<char> = Vec::new();
                    let mut terminated = false;
                    for c in chars.by_ref() {
                        if c == '"' {
                            terminated = true;
                            break;
                        }
                        if c == '\n' {
                            line += 1;
                        }
                        str_chars.push(c);
                    }

                    if !terminated {
                        tracing::error!("Unterminated string");
                        had_error = true;
                        break;
                    }

                    let string_value: String = str_chars.iter().collect();
                    tokens.push(Token::new(
                        TokenType::String(string_value.clone()),
                        format!("\"{}\"", string_value),
                        line,
                    ));
                }
                _ if c.is_ascii_digit() => match parse_digit(&mut chars, c, line) {
                    Ok(token) => tokens.push(token),
                    Err(e) => {
                        tracing::error!("{}", e);
                        had_error = true;
                    }
                },
                _ if c.is_ascii_alphabetic() => match parse_identifier(&mut chars, c, line) {
                    Ok(token) => tokens.push(token),
                    Err(e) => {
                        tracing::error!("{}", e);
                        had_error = true;
                    }
                },
                _ => {
                    tracing::error!("Unexpected character '{}' on line {}", c, line);
                    had_error = true;
                }
            }
        }

        if had_error {
            bail!("Invalid syntax")
        }
        Ok(tokens)
    }
}

fn parse_identifier(
    chars: &mut Peekable<Chars<'_>>,
    first: char,
    line: u64,
) -> anyhow::Result<Token> {
    let mut identifier: Vec<char> = vec![first];
    while let Some(c) = chars.next() {
        identifier.push(c);
        if !chars
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
        {
            break;
        }
    }

    let identifier: String = identifier.iter().collect();
    let token_type =
        TokenType::from_keyword(&identifier).unwrap_or(TokenType::Identifier(identifier.clone()));

    Ok(Token::new(token_type, identifier, line))
}

fn parse_digit(chars: &mut Peekable<Chars<'_>>, first: char, line: u64) -> anyhow::Result<Token> {
    let mut digits: Vec<char> = vec![first];
    fn is_next_valid(chars: &mut Peekable<Chars<'_>>) -> bool {
        // if next is a digit we can consume it
        if chars.peek().is_some_and(|c| c.is_ascii_digit()) {
            return true;
        }

        if chars.peek().is_some_and(|c| *c == '.') {
            let mut chars_clone = chars.clone();
            // advance over the '.'
            chars_clone.next();
            chars_clone.next().is_some_and(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    while is_next_valid(chars)
        && let Some(c) = chars.by_ref().next()
    {
        digits.push(c);
    }

    let n_str = digits.iter().collect::<String>();
    let n = n_str
        .parse::<f64>()
        .expect("This should always be a valid float");
    Ok(Token::new(TokenType::Number(n), n_str, line))
}
