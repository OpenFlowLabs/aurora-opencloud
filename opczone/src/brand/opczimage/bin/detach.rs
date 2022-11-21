use anyhow::Result;
use clap::Parser;
use common::init_slog_logging;
use std::{fs::copy, path::Path};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'z')]
    zonename: String,

    #[arg(short = 'R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let cli: Cli = Cli::parse();

    // cp /etc/zones/${ZONENAME}.xml ${ZONEPATH}/SUNWdetached.xml
    copy(
        Path::new("/etc/zones").join(format!("{}.xml", &cli.zonename)),
        Path::new(&cli.zonepath).join("SUNWdetached.xml"),
    )?;

    Ok(())
}
