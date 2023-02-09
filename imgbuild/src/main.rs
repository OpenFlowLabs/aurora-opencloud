use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use common::{debug, info, init_slog_logging};
use opczone::brand::Brand;
use opczone::get_zone_dataset;
use opczone::image::{export_image_as_dataset_format, export_zone_as_oci_format};
use opczone::machine::AddNicPayload;
use opczone::{brand::build_zonecontrol_gz_path, machine::define_vm};
use std::fmt::Display;
use std::fs::{DirBuilder, File};
use std::io::Write;
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
    #[command(subcommand)]
    commands: Commands,
}

#[derive(ValueEnum, Clone)]
enum ExportType {
    Dataset,
    OCI,
}

impl Default for ExportType {
    fn default() -> Self {
        Self::Dataset
    }
}

impl Display for ExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportType::Dataset => write!(f, "dataset image"),
            ExportType::OCI => write!(f, "oci image"),
        }
    }
}

/// All build commands
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new build bundle
    Init {
        /// Optionally define the Path where the new image should be initialized assumes CWD by default
        location: Option<PathBuf>,

        /// With what name to initialize the image
        #[arg(short, long)]
        name: Option<String>,

        /// Define the author of the image already during initialization
        #[arg(short, long)]
        author: Option<String>,
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

        #[arg(short, long)]
        /// Define the default router
        gateway: Option<String>,

        #[arg(short = 'e', long, default_value = "dataset")]
        /// To which image format to export into
        image_export_type: ExportType,

        /// Tell the Cli the location of the build bundle. Assumes CWD as default
        build_bundle: Option<String>,
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
        Commands::Init {
            location,
            name,
            author,
        } => {
            let location = std::fs::canonicalize(if let Some(location) = location {
                if !location.exists() {
                    debug!("Creating bundle directory {}", &location.display());
                    DirBuilder::new()
                        .recursive(true)
                        .create(&location)
                        .context("could not create bundle directory")?;
                }

                location
            } else {
                Path::new(".").to_path_buf()
            })?;

            let mut doc = kdl::KdlDocument::new();
            if let Some(name) = name {
                let mut name_node = kdl::KdlNode::new("name");
                name_node.push(kdl::KdlEntry::new(name));
                doc.nodes_mut().push(name_node);
            }

            if let Some(author) = author {
                let mut author_node = kdl::KdlNode::new("author");
                author_node.push(kdl::KdlEntry::new(author));
                doc.nodes_mut().push(author_node);
            }

            debug!("location is: {}", &location.display());

            debug!("writing build.kdl");
            let mut build_kdl = File::create(&location.join("build.kdl"))
                .with_context(|| "could not write build.kdl")?;
            build_kdl.write_all(&doc.to_string().as_bytes())?;
        }

        Commands::Build {
            nictag,
            quota,
            ram,
            ip,
            gateway,
            build_bundle,
            image_export_type,
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
                    gateway,
                    ..Default::default()
                });

                cfg.nics = Some(nics)
            }

            let quota = cfg.quota.clone();

            let conf = define_vm(cfg)?;
            let zonename = conf.uuid.to_string();

            let mut zoneadm = zone::Adm::new(&zonename);

            let bundle = std::fs::canonicalize(if let Some(build_bundle) = build_bundle {
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
            let zone_ds_name = get_zone_dataset(&zone_path.as_os_str().to_string_lossy())?;
            let mut zonecfg_zone = zone::Config::new(&zonename);
            zonecfg_zone.add_dataset(&zone::Dataset {
                name: format!("{}/vroot", zone_ds_name),
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

            let image_uuid = opczone::image::convert_zone_to_image(&zonename)?;

            match image_export_type {
                ExportType::Dataset => export_image_as_dataset_format(image_uuid, output_dir)?,
                ExportType::OCI => export_zone_as_oci_format(zone, output_dir)?,
            }
        }
        #[allow(unused_variables)]
        Commands::Publish { endpoint } => {}
    }
    Ok(())
}
