use std::{path::Path, fs::{DirBuilder, File}, os::unix::fs::DirBuilderExt};

use common::{init_slog_logging};
use anyhow::{Result, Context};
use clap::{Parser};
use opczone::get_zonepath_parent_ds;
use zone::{Config as ZoneConfig, Global};
use illumos_image_builder::{dataset_create, zfs_set, dataset_clone};

const INSTALL_ZONE_CONFIG_PATH: &str = "/etc/zones/install";

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

    let cfg = get_install_config(&cli.zonename)?;

    setup_dataset(&cli.zonename, &cli.zonepath, cfg.quota, cfg.image)?;

    setup_zone_fs(&cli.zonename, &cli.zonepath)?;

    Ok(())
}

fn get_install_config(zonename: &str) -> Result<opczone::Config> {
    let config_path = Path::new(INSTALL_ZONE_CONFIG_PATH).join(format!("{}.json", zonename));

    let cfg_file = File::open(config_path)?;

    let cfg = serde_json::from_reader(cfg_file)?;

    Ok(cfg)
}


/// For image based zones we clone the image as a new zone.
/// if we are building a new zone, we create new datasets for the zone completely empty 
fn setup_dataset(zonename: &str, zonepath: &str, zonequota: i32, image: Option<uuid::Uuid>) -> Result<()> {
    // We expect that zoneadm was invoked with '-x nodataset', so it won't have
	// created the dataset.
    let parent_dataset = get_zonepath_parent_ds(zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_dataset, zonename);
    let root_dataset_name = format!("{}/{}", zone_dataset_name, "root");

    let quota_arg = format!("{}g", zonequota);

    // Create the dataset where we will keep the configurations and all needed metadata for the zone
    dataset_create(&zone_dataset_name, false)?;

    if let Some(image) = image {
        let snapshot = format!("{}/{}@final", parent_dataset, image.as_hyphenated().to_string());
        let quota_opt = format!("quota={}", quota_arg);
        dataset_clone(&snapshot, &root_dataset_name, true, Some(vec![
            "devices=off".into(), 
            quota_opt,
        ]))?;
    } else {
        dataset_create(&root_dataset_name, false)?;
        zfs_set(&root_dataset_name, "devices", "off")?;
        zfs_set(&root_dataset_name, "quota", &quota_arg)?;
    }

    Ok(())
}

fn setup_zone_fs(zonename: &str, zonepath: &str) -> Result<()> {
    let config_path = Path::new(zonepath).join("config");
    let zone_root = Path::new(zonepath).join("root");
    
    if !config_path.exists() {
        DirBuilder::new().mode(0o755).create(&config_path)
        .context(format!("unable to create zone config directory: {}", zonename))?;
    }

    let zone_tmp = zone_root.join("tmp");
    if !zone_tmp.exists() {
        DirBuilder::new().mode(0o1777).create(&zone_tmp)
        .context(format!("unable to create zone tmp directory: {}", zonename))?;
    }

    Ok(())
}