use std::fs::File;
use std::path::Path;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use common::init_slog_logging;
use opczone::{build::{add_build_config, Bundle}, machine::define_vm};
use std::io::stdin;
use url::Url;

const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";

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

        /// Define a build bundle to make avaialble in the zone. This also sets up build assistance tools
        #[clap(short)]
        build_bundle_fmri: Option<String>,
    },
    /// Delete the VM with the specified UUID. The VM and any associated
    /// storage including zvols and the zone filesystem will be removed.
    Delete {
        #[clap(value_parser)]
        uuid: uuid::Uuid,
    },
    /// The list command can list the VMs on a system in a variety of ways.
    List {
        #[clap(short)]
        order: Option<String>,

        #[clap(short)]
        sort: Option<String>,

        #[clap(short, default_value = "true")]
        parseable: bool,

        #[clap(short = 'H', default_value = "true")]
        header: bool,

        #[clap(value_parser)]
        filter: Vec<String>,
    },
}

fn main() -> Result<()> {
    let _logger_guard = init_slog_logging(false)?;

    let cli: Cli = Cli::parse();

    match cli.command {
        Subcommands::Create { filename, build_bundle_fmri} => {
            let mut cfg: opczone::machine::CreatePayload = if let Some(filename) = filename {
                let file = File::open(&filename)
                    .context(format!("could not open payload file {}", &filename))?;

                serde_json::from_reader(file)?
            } else {
                serde_json::from_reader(stdin())?
            };

            let quota = cfg.quota.clone();

            let build_args = if let Some(build_bundle_fmri) = build_bundle_fmri {
                // TODO: handle non dir build bundle FMRI's
                let bundle_url: Url = build_bundle_fmri.parse()?;
                let bundle = match Bundle::new(Path::new(bundle_url.path())) {
                    Ok(v)=> v,
                    Err(err) => {
                        println!("{:?}", err);
                        std::process::exit(1);
                    }
                };

                Some(vec![
                    "-b".to_owned(),
                    bundle.get_path().to_string_lossy().to_string(),
                ])
            } else {
                None
            };

            let conf = define_vm(cfg)?;

            let mut zoneadm: Vec<String> = vec![
                ZONEADM_BIN.into(), 
                "-z".into(), 
                conf.uuid.to_string(), 
                "install".into(),
                "-q".into(),
                quota.to_string(),
            ];

            zoneadm = if let Some(build_args) = build_args {
                vec![zoneadm, build_args].concat()
            } else {
                zoneadm
            };

            opczone::run(&zoneadm, None)?;
        }
        Subcommands::Delete { uuid } => todo!(),
        Subcommands::List {
            order,
            sort,
            parseable,
            header,
            filter,
        } => todo!(),
    }

    Ok(())
}
