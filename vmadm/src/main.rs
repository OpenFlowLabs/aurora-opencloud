use std::fs::DirBuilder;
use std::{fs::File};
use std::path::Path;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use common::{init_slog_logging, info, debug, trace};
use opczone::{build::bundle::Bundle, machine::define_vm, brand::build_zonecontrol_gz_path};
use std::io::stdin;

const ZONEADM: &str = "/usr/sbin/zoneadm";
const ZLOGIN: &str = "/usr/sbin/zlogin";

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Create a new VM on the system. Any images/datasets referenced must
    /// already exist on the target zpool.
    Create {
        #[clap(short)]
        filename: Option<String>,
    },
    /// Delete the VM with the specified UUID. The VM and any associated
    /// storage including zvols and the zone filesystem will be removed.
    Delete {
        #[clap(value_parser)]
        _uuid: uuid::Uuid,
    },
    /// The list command can list the VMs on a system in a variety of ways.
    List {
        #[clap(short)]
        _order: Option<String>,

        #[clap(short)]
        _sort: Option<String>,

        #[clap(short, default_value = "true")]
        _parseable: bool,

        #[clap(short = 'H', default_value = "true")]
        _header: bool,

        #[clap(value_parser)]
        _filter: Vec<String>,
    },
}

fn main() -> Result<()> {
    let _logger_guard = init_slog_logging(false, false)?;

    let cli: Cli = Cli::parse();

    match cli.command {
        Subcommands::Create { filename, build_bundle_fmri, export_to_image} => {
            let cfg: opczone::machine::CreatePayload = if let Some(filename) = filename {
                let file = File::open(&filename)
                    .context(format!("could not open payload file {}", &filename))?;

                serde_json::from_reader(file)?
            } else {
                serde_json::from_reader(stdin())?
            };

            println!("Zone: {:#?}", cfg);
        }
        Subcommands::Delete { _uuid } => todo!(),
        Subcommands::List {
            _order,
            _sort,
            _parseable,
            _header,
            _filter,
        } => {
            let zlist = zone::Adm::list()?;
            println!("{:?}", zlist);
        },
    }

    Ok(())
}
