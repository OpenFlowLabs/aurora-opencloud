use clap::Parser;
use common::{init_slog_logging, warn};
use miette::{bail, IntoDiagnostic, Result};
use opczone::get_zonepath_parent_ds;
use std::process::Command;
use std::{fs::remove_dir_all, path::Path};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'z')]
    zonename: String,

    #[arg(short = 'R')]
    zonepath: String,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let cli: Cli = Cli::parse();

    let parent_ds = get_zonepath_parent_ds(&cli.zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_ds, &cli.zonename);

    let _root_dataset_name = format!("{}/{}/root", parent_ds, &cli.zonename);

    let zone_control_dir = format!("/var/zonecontrol/{}", &cli.zonename);

    let zone_ds = solarm_utils::zfs::open(&zone_dataset_name).into_diagnostic()?;

    match zone_ds.destroy() {
        Ok(_) => {}
        Err(_) => {
            warn!("DESTROY FAILED trying again forced");
            let zfs = Command::new("/sbin/zfs")
                .env_clear()
                .arg("destroy")
                .arg("-rfF")
                .arg(&zone_dataset_name)
                .output()
                .into_diagnostic()?;
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
        remove_dir_all(zonepath).into_diagnostic()?;
    }

    let zone_control_path = Path::new(&zone_control_dir);
    if zone_control_path.exists() {
        remove_dir_all(zone_control_path).into_diagnostic()?;
    }

    Ok(())
}
