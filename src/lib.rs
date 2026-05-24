use std::{
    fs::read_to_string,
    io::{BufRead, BufReader, Write, stdin, stdout},
    path::Path,
};

use tracing::info;

use crate::{expr::evaluate, parser::TokenParser, token::TokenScanner};

mod ast_pretty_print;
mod expr;
mod parser;
mod token;

pub fn run_file(source: &Path) -> anyhow::Result<()> {
    let contents = read_to_string(source)?;
    let mut scanner = TokenScanner::default();
    run(&mut scanner, &contents)
}

pub fn run_prompt() -> anyhow::Result<()> {
    let mut reader = BufReader::new(stdin());
    let mut scanner = TokenScanner::default();
    loop {
        print!("> ");
        stdout().flush()?;
        let mut line = String::new();
        let nread = reader.read_line(&mut line)?;
        if nread == 0 {
            break;
        }

        if let Err(e) = run(&mut scanner, &line) {
            eprintln!("{}", e);
        }
    }

    Ok(())
}

pub fn run(scanner: &mut TokenScanner, source: &str) -> anyhow::Result<()> {
    let tokens = scanner.scan_tokens(source)?;
    for token in &tokens {
        info!("{:?}", token);
    }

    let mut parser = TokenParser::new(tokens.iter().peekable());
    if let Some(expr) = parser.parse() {
        info!("{}", expr.pretty_print_ast()?);
        println!("{:?}", evaluate(expr)?);
    } else {
        info!("Invalid Syntax");
    }
    Ok(())
}
