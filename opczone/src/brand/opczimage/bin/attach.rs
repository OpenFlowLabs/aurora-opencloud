use anyhow::Result;
use clap::Parser;
use common::init_slog_logging;

#[derive(Parser)]
struct Cli {
    #[arg(short = 'z')]
    zonename: String,

    #[arg(short = 'R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let _cli: Cli = Cli::parse();

    Ok(())
}
