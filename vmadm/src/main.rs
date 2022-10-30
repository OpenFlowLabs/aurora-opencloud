use std::fs::DirBuilder;
use std::{fs::File};
use std::path::Path;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use common::{init_slog_logging, info, debug, trace};
use opczone::{build::bundle::Bundle, machine::define_vm, brand::build_zonecontrol_gz_path};
use std::io::stdin;

const RUNNER_BRAND_PATH: &str = "/usr/lib/brand/opczimage/build_runner";
const RUNNER_IN_ZONE_PATH_RELATIVE: &str = "build_runner";
const RUNNER_IN_ZONE_PATH_ABSOLUTE: &str = "/build_runner";
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

        /// Define a build bundle to make avaialble in the zone. This also sets up build assistance tools
        #[clap(short)]
        build_bundle_fmri: Option<String>,

        ///Export the created zone directly as image
        #[clap(short)]
        export_to_image: bool,
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

            let quota = cfg.quota.clone();

            let conf = define_vm(cfg)?;
            let zonename = conf.uuid.to_string();

            let mut zoneadm = zone::Adm::new(&zonename);

            // We use opczone::run here to install the zone because the zone package gets all output before 
            // returning it to stdout. opczone::run shows progress immediatly
            if let Some(b) = build_bundle_fmri {
                opczone::run(&[
                    ZONEADM,
                    "-z",
                    &zonename,
                    "install",
                    "-q",
                    &quota.to_string(),
                    "-b",
                    &b,
                ], None)?;
            } else {
                opczone::run(&[
                    ZONEADM,
                    "-z",
                    &zonename,
                    "install",
                    "-q",
                    &quota.to_string(),
                ], None)?;
            }

            //Boot Zone
            zoneadm.boot()?;
            trace!("all zones on the system: {:#?}", zone::Adm::list());
            debug!("trying to get zonepath of {}", zonename);
            let zone = opczone::get_zone(&zonename)?;

            //Copy Builder into zone
            let zone_path = zone.path();
            debug!("Zone path: {}", zone_path.display());
            let gz_runner_in_zone_path = zone_path.join("root").join(RUNNER_IN_ZONE_PATH_RELATIVE);

            info!("copying build_runner into zone {}", zone.name());
            debug!("{} -> {}", RUNNER_BRAND_PATH, gz_runner_in_zone_path.display());
            fs_extra::file::copy(RUNNER_BRAND_PATH, &gz_runner_in_zone_path, &fs_extra::file::CopyOptions{
                skip_exist: true,
                ..Default::default()
            })?;

            //Run Builder inside zone with zlogin
            //we again use opczone::run to get all the output
            opczone::run(&[
                ZLOGIN,
                //"-Q",
                &zonename,
                RUNNER_IN_ZONE_PATH_ABSOLUTE,
            ], None)?;

            //Cleanup Bundle
            let bundle_zonecontrol_path = build_zonecontrol_gz_path(&zonename).join("build_bundle");
            let cleanup_items = vec![bundle_zonecontrol_path.as_path(), &gz_runner_in_zone_path];
            fs_extra::remove_items(&cleanup_items)?;

            //TODO: Run export if export_to_image is set
            if export_to_image {
                
            }
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
