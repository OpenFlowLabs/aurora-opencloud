use std::{fs::{copy, remove_dir_all}, path::Path};

use anyhow::bail;
use clap::{Parser};
use common::{warn, init_slog_logging};
use illumos_image_builder::{zfs_get, dataset_remove};
use opczone::get_zonepath_parent_ds;

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

    let parent_ds = get_zonepath_parent_ds(&cli.zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_ds, &cli.zonename);

    let root_dataset_name = format!("{}/{}/root", parent_ds, &cli.zonename);

    let zone_control_dir = format!("/var/zonecontrol/{}", &cli.zonename);
    
    match dataset_remove(&zone_dataset_name) {
        Ok(_) => {},
        Err(err) => {
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
                    return Ok(false);
                }
                bail!("zfs destroy failed: {}", errmsg);
            }
        }
    }

    remove_dir_all(&cli.zonepath)?;
    remove_dir_all(&zone_control_dir)?;

    Ok(())
}
