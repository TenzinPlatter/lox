use std::path::PathBuf;

use anyhow::bail;
use lox::{run_file, run_prompt};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() > 2 {
        bail!("Usage: lox [script]");
    }

    if let Some(script) = args.get(1) {
        let path = PathBuf::from(script);
        run_file(&path)?;
    } else {
        run_prompt()?;
    };

    Ok(())
}
