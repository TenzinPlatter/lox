use std::path::PathBuf;

use anyhow::bail;
use lox::{run_file, run_prompt};
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> anyhow::Result<()> {
    fmt()
        .with_writer(std::io::stdout)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

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
