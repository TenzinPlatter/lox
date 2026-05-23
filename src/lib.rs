use std::{
    fs::read_to_string,
    io::{BufRead, BufReader, stdin},
    path::Path,
};

use crate::token::parse_tokens;

mod token;

pub fn run_file(source: &Path) -> anyhow::Result<()> {
    let contents = read_to_string(source)?;
    run(&contents)
}

pub fn run_prompt() -> anyhow::Result<()> {
    let mut reader = BufReader::new(stdin());
    loop {
        print!("> ");
        let mut line = String::new();
        let nread = reader.read_line(&mut line)?;
        if nread == 0 {
            break;
        }

        if let Err(e) = run(&line) {
            eprintln!("{}", e);
        }
    }

    Ok(())
}

pub fn run(source: &str) -> anyhow::Result<()> {
    let tokens = parse_tokens(source)?;
    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}

