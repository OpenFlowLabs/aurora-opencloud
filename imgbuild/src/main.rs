use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use common::{debug, info, init_slog_logging, trace};
use opczone::brand::Brand;
use opczone::machine::AddNicPayload;
use opczone::{brand::build_zonecontrol_gz_path, build::bundle::Bundle, machine::define_vm};
use std::fs::DirBuilder;
use std::fs::File;
use std::io::stdin;
use std::path::Path;

const RUNNER_BRAND_PATH: &str = "/usr/lib/brand/opczimage/build_runner";
const RUNNER_IN_ZONE_PATH_RELATIVE: &str = "build_runner";
const RUNNER_IN_ZONE_PATH_ABSOLUTE: &str = "/build_runner";
const ZONEADM: &str = "/usr/sbin/zoneadm";
const ZLOGIN: &str = "/usr/sbin/zlogin";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Define a build bundle to make avaialble in the zone. This also sets up build assistance tools
    build_bundle_fmri: String,

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
    ip: Option<String>
}

fn main() -> Result<()> {
    let _logger_guard = init_slog_logging(false, false)?;

    let cli: Cli = Cli::parse();

    //TODO: extract some definitions of networking from some
    // textfiles and setup the zone with enough data automatically

    let mut cfg = opczone::machine::CreatePayload {
        brand: Brand::Image,
        zfs_io_priority: 30,
        ram: cli.ram,
        quota: cli.quota,
        max_physical_memory: Some(cli.ram),
        ..Default::default()
    };

    if let Some(nictag) = cli.nictag {
        let mut nics = if let Some(nics) = cfg.nics {
            nics
        } else {
            vec![]
        };

        nics.push(AddNicPayload{
            nic_tag: Some(nictag),
            ip: cli.ip,
            ..Default::default()
        });

        cfg.nics = Some(nics)
    }

    let quota = cfg.quota.clone();

    let conf = define_vm(cfg)?;
    let zonename = conf.uuid.to_string();

    let mut zoneadm = zone::Adm::new(&zonename);

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
            &cli.build_bundle_fmri,
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
        &[
            ZLOGIN,
            "-Q",
            &zonename,
            RUNNER_IN_ZONE_PATH_ABSOLUTE,
        ],
        None,
    )?;

    //Cleanup Bundle
    let bundle_zonecontrol_path = build_zonecontrol_gz_path(&zonename).join("build_bundle");
    let cleanup_items = vec![bundle_zonecontrol_path.as_path(), &gz_runner_in_zone_path];
    fs_extra::remove_items(&cleanup_items)?;

    let output_dir = std::env::current_dir()?;

    opczone::image::convert_zone_to_image(&zonename, 
        &output_dir, opczone::image::ImageType::Dataset)?;

    Ok(())
}
