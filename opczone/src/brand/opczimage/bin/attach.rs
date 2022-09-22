use anyhow::Result;
use clap::Parser;
use common::init_slog_logging;

#[derive(Parser)]
struct Cli {
    #[clap(short = 'z')]
    zonename: String,

    #[clap(short = 'R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false)?;

    let _cli: Cli = Cli::parse();

    Ok(())
}
