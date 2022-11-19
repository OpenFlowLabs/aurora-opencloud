use anyhow::Result;
use clap::{Parser, ArgEnum};

#[derive(ArgEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum Command {
    Datasets,
    Skip,
}

#[derive(Parser)]
struct Cli {
    #[clap(value_parser)]
    zonename: String,

    #[clap(value_parser)]
    zonepath: String,

    #[clap(value_parser, value_enum, default_value_t=Command::Skip)]
    command: Command,
}

fn main() -> Result<()> {

    let zero = char::from(0);

    let cli: Cli = Cli::parse();

    match cli.command {
        Command::Datasets => {
            print!("{}/vroot", &cli.zonepath);
            print!("{}/root", &cli.zonepath);
        },
        Command::Skip => {},
    } 

    Ok(())
}
