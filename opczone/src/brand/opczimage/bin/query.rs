use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum Command {
    Datasets,
    Skip,
}

#[derive(Parser)]
struct Cli {
    #[arg(value_parser)]
    zonename: String,

    #[arg(value_parser)]
    zonepath: String,

    #[arg(value_enum, default_value_t=Command::Skip)]
    command: Command,
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Command::Datasets => {
            print!("{}/vroot", &cli.zonepath);
            print!("{}/root", &cli.zonepath);
        }
        Command::Skip => {}
    }

    Ok(())
}
