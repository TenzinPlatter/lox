use std::{
    fs::read_to_string,
    io::{BufRead, BufReader, Write, stdin, stdout},
    path::Path,
};

use crate::token::{Token, TokenParser};

mod token;

pub fn run_file(source: &Path) -> anyhow::Result<()> {
    let contents = read_to_string(source)?;
    let mut parser = TokenParser::default();
    run(&mut parser, &contents)
}

pub fn run_prompt() -> anyhow::Result<()> {
    let mut reader = BufReader::new(stdin());
    let mut parser = TokenParser::default();
    loop {
        print!("> ");
        stdout().flush()?;
        let mut line = String::new();
        let nread = reader.read_line(&mut line)?;
        if nread == 0 {
            break;
        }

        if let Err(e) = run(&mut parser, &line) {
            eprintln!("{}", e);
        }
    }

    Ok(())
}

pub fn run(parser: &mut TokenParser, source: &str) -> anyhow::Result<()> {
    let tokens = parser.parse_tokens(source)?;
    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}
