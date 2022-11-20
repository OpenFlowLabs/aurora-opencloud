use anyhow::Result;
use clap::{Parser, Subcommand};
use common::{debug, info, init_slog_logging};
use opczone::brand::Brand;
use opczone::machine::AddNicPayload;
use opczone::{brand::build_zonecontrol_gz_path, machine::define_vm};
use std::path::{Path, PathBuf};
use url::Url;

const RUNNER_BRAND_PATH: &str = "/usr/lib/brand/opczimage/build_runner";
const RUNNER_IN_ZONE_PATH_RELATIVE: &str = "build_runner";
const RUNNER_IN_ZONE_PATH_ABSOLUTE: &str = "/build_runner";
const ZONEADM: &str = "/usr/sbin/zoneadm";
const ZLOGIN: &str = "/usr/sbin/zlogin";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Tell the Cli the location of the build bundle. Assumes CWD as default
    build_bundle: Option<String>,

    #[command(subcommand)]
    commands: Commands,
}

/// All build commands
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new build bundle
    Init {
        /// Optionally define the Path where the new image should be initialized assumes CWD by default
        location: Option<PathBuf>,
    },
    Build {
        #[arg(short, long)]
        /// Define the network interface or etherstub or overlay to bind the zone to.
        nictag: Option<String>,

        #[arg(short, long, default_value_t = 20)]
        /// Disk Quota of zone during building
        quota: u32,

        #[arg(short, long, default_value_t = 1024)]
        /// RAM allocated during building in MiB
        ram: u32,

        #[arg(short, long)]
        /// Define the IP address the Build zone will use.
        ip: Option<String>,
    },
    Publish {
        /// Tell the utility where to publish the image to. Use oci:// for OCI registry endpoints
        endpoint: Url,
    },
}

fn main() -> Result<()> {
    let _logger_guard = init_slog_logging(false, false)?;

    let cli: Cli = Cli::parse();

    //TODO: extract some definitions of networking from some
    // textfiles and setup the zone with enough data automatically
    match cli.commands {
        Commands::Init { location } => {}
        Commands::Build {
            nictag,
            quota,
            ram,
            ip,
        } => {
            let mut cfg = opczone::machine::CreatePayload {
                brand: Brand::Image,
                zfs_io_priority: 30,
                ram,
                quota,
                max_physical_memory: Some(ram),
                ..Default::default()
            };

            if let Some(nictag) = nictag {
                let mut nics = if let Some(nics) = cfg.nics {
                    nics
                } else {
                    vec![]
                };

                nics.push(AddNicPayload {
                    nic_tag: Some(nictag),
                    ip,
                    ..Default::default()
                });

                cfg.nics = Some(nics)
            }

            let quota = cfg.quota.clone();

            let conf = define_vm(cfg)?;
            let zonename = conf.uuid.to_string();

            let mut zoneadm = zone::Adm::new(&zonename);

            let bundle = std::fs::canonicalize(if let Some(build_bundle) = cli.build_bundle {
                Path::new(build_bundle.as_str()).to_path_buf()
            } else {
                Path::new(".").to_path_buf()
            })?;

            // We use opczone::run here to install the zone because the zone package gets all output before
            // returning it to stdout. opczone::run shows progress immediatly
            opczone::run(
                &[
                    ZONEADM,
                    "-z",
                    &zonename,
                    "install",
                    "-q",
                    &quota.to_string(),
                    "-b",
                    bundle.to_str().expect("non UTF-8 paths can not be used by this program please put the bundle somewhere where there is UTF-8"),
                ],
                None,
            )?;

            let zone = opczone::get_zone(&zonename)?;
            debug!("trying to get zonepath of {}", zonename);
            let zone_path = zone.path();
            debug!("Zone path: {}", zone_path.display());

            //Add Volume root to delegated dataset
            let mut zonecfg_zone = zone::Config::new(&zonename);
            zonecfg_zone.add_dataset(&zone::Dataset {
                name: format!("{}/vroot", zone_path.as_os_str().to_string_lossy()),
            });
            let out = zonecfg_zone.run()?;
            info!("Updating zone config: {}", out);

            //Boot Zone
            zoneadm.boot()?;

            //Copy Builder into zone
            let gz_runner_in_zone_path = zone_path.join("root").join(RUNNER_IN_ZONE_PATH_RELATIVE);

            info!("copying build_runner into zone {}", zone.name());
            debug!(
                "{} -> {}",
                RUNNER_BRAND_PATH,
                gz_runner_in_zone_path.display()
            );
            fs_extra::file::copy(
                RUNNER_BRAND_PATH,
                &gz_runner_in_zone_path,
                &fs_extra::file::CopyOptions {
                    skip_exist: true,
                    ..Default::default()
                },
            )?;

            //Run Builder inside zone with zlogin
            //we again use opczone::run to get all the output
            opczone::run(
                &[ZLOGIN, "-Q", &zonename, RUNNER_IN_ZONE_PATH_ABSOLUTE],
                None,
            )?;

            //Cleanup Bundle
            let bundle_zonecontrol_path = build_zonecontrol_gz_path(&zonename).join("build_bundle");
            let cleanup_items = vec![bundle_zonecontrol_path.as_path(), &gz_runner_in_zone_path];
            fs_extra::remove_items(&cleanup_items)?;

            let output_dir = std::env::current_dir()?;

            opczone::image::convert_zone_to_image(
                &zonename,
                &output_dir,
                opczone::image::ImageType::Dataset,
            )?;
        }
        Commands::Publish { endpoint } => {}
    }
    Ok(())
}
