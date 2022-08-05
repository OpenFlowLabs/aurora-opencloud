use std::{fs::copy, path::Path};
use common::{init_slog_logging};
use clap::{Parser};
use anyhow::Result;

#[derive(Parser)]
struct Cli {
    #[clap(short='z')]
    zonename: String,

    #[clap(short='R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false)?;

    let cli: Cli = Cli::parse();

    // cp /etc/zones/${ZONENAME}.xml ${ZONEPATH}/SUNWdetached.xml
    copy(
        Path::new("/etc/zones").join(format!("{}.xml", &cli.zonename)), 
        Path::new(&cli.zonepath).join("SUNWdetached.xml")
    )?;

    Ok(())
}