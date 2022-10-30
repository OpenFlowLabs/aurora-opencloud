use std::{fs::remove_dir_all, path::Path};

use anyhow::{bail, Result};
use clap::Parser;
use common::{init_slog_logging, warn};
use illumos_image_builder::dataset_remove;
use opczone::get_zonepath_parent_ds;
use std::process::Command;

#[derive(Parser)]
struct Cli {
    #[clap(short = 'z')]
    zonename: String,

    #[clap(short = 'R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let cli: Cli = Cli::parse();

    let parent_ds = get_zonepath_parent_ds(&cli.zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_ds, &cli.zonename);

    let _root_dataset_name = format!("{}/{}/root", parent_ds, &cli.zonename);

    let zone_control_dir = format!("/var/zonecontrol/{}", &cli.zonename);

    match dataset_remove(&zone_dataset_name) {
        Ok(_) => {}
        Err(_) => {
            warn!("DESTROY FAILED trying again forced");
            let zfs = Command::new("/sbin/zfs")
                .env_clear()
                .arg("destroy")
                .arg("-rfF")
                .arg(&zone_dataset_name)
                .output()?;
            if !zfs.status.success() {
                let errmsg = String::from_utf8_lossy(&zfs.stderr);
                if errmsg.trim().ends_with("dataset does not exist") {
                    return Ok(());
                }
                bail!("zfs destroy failed: {}", errmsg);
            }
        }
    }

    let zonepath = Path::new(&cli.zonepath);
    if zonepath.exists() {
        remove_dir_all(zonepath)?;
    }

    let zone_control_path = Path::new(&zone_control_dir);
    if zone_control_path.exists() {
        remove_dir_all(zone_control_path)?;
    }

    Ok(())
}
