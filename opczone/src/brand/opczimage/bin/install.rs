use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use common::init_slog_logging;
use illumos_image_builder::{dataset_clone, dataset_create, zfs_set};
use opczone::{
    build::{bundle::{Bundle}, run_action},
    get_zonepath_parent_ds,
    vmext::get_brand_config,
};
use std::{
    fs::DirBuilder,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};
//use zone::{Config as ZoneConfig, Global};

#[derive(Parser)]
struct Cli {
    #[clap(short = 'z')]
    zonename: String,

    #[clap(short = 'R')]
    zonepath: String,

    #[clap(short = 'q')]
    quota: i32,

    #[clap(short = 't')]
    image_uuid: Option<uuid::Uuid>,

    #[clap(short = 'b')]
    build_bundle: Option<PathBuf>,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false)?;

    let cli: Cli = Cli::parse();

    let _cfg = get_brand_config(&cli.zonename)?;

    if cli.image_uuid.is_some() && cli.build_bundle.is_some() {
        bail!("can only either deploy an image production by setting and image or build an image by setting build bundle. Both are set, bailing")
    }

    setup_dataset(
        &cli.zonename,
        &cli.zonepath,
        cli.quota,
        cli.image_uuid,
        cli.build_bundle.clone(),
    )?;

    if let Some(build_bundle) = cli.build_bundle {
        let bundle = Bundle::new(&build_bundle).map_err(|err| anyhow!("{:?}", err))?;
        let bundle_audit = bundle.get_audit_info();
        if !bundle_audit.is_base_image() {
            bail!("Bundle is not safe to run in gz: Either this bundle must be based on another image or it's first action must be an ips action.")
        }

        let zone_root = format!("{}/root", cli.zonepath);

        //Run first IPS action to install image base
        if let Some(ips_action) = bundle.document.actions.first() {
            run_action(&zone_root, ips_action.clone())?;
        }

        //Save image bundle inside the image with first IPS action removed
        bundle.save_to_zone(&zone_root)?;
    }

    setup_zone_fs(&cli.zonename, &cli.zonepath)?;

    Ok(())
}

/// For image based zones we clone the image as a new zone.
/// if we are building a new zone, we create new datasets for the zone completely empty
fn setup_dataset(
    zonename: &str,
    zonepath: &str,
    zonequota: i32,
    image: Option<uuid::Uuid>,
    build_bundle: Option<PathBuf>,
) -> Result<()> {
    let parent_dataset = get_zonepath_parent_ds(zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_dataset, zonename);
    let root_dataset_name = format!("{}/{}", zone_dataset_name, "root");

    let quota_arg = format!("{}g", zonequota);

    // zoneadm already creates a dataset for the zone and does not swallow a -x argument
    // thus we do not need to create the top level dataset
    // dataset_create(&zone_dataset_name, false)?;

    if let Some(image) = image {
        let snapshot = format!("{}/{}@final", parent_dataset, image.to_string());
        let quota_opt = format!("quota={}", quota_arg);
        dataset_clone(
            &snapshot,
            &root_dataset_name,
            true,
            Some(vec!["devices=off".into(), quota_opt]),
        )?;
    } else if let Some(bundle_path) = build_bundle {
        let bundle = Bundle::new(&bundle_path).map_err(|err| anyhow!("{:?}", err))?;
        let audit_info = bundle.get_audit_info();
        if audit_info.is_base_image() {
            dataset_create(&root_dataset_name, false)?;
            zfs_set(&root_dataset_name, "devices", "off")?;
            zfs_set(&root_dataset_name, "quota", &quota_arg)?;
        } else {
            //TODO: clone base image
            todo!()
        }
    } else {
        bail!("neither image uuid or build bundle specified this would create an empty (unusable) zone")
    }

    Ok(())
}

fn setup_zone_fs(zonename: &str, zonepath: &str) -> Result<()> {
    let config_path = Path::new(zonepath).join("config");
    let zone_root = Path::new(zonepath).join("root");
    let zone_tmp = zone_root.join("tmp");

    if !config_path.exists() {
        DirBuilder::new()
            .mode(0o755)
            .create(&config_path)
            .context(format!(
                "unable to create zone config directory: {}",
                zonename
            ))?;
    }

    if !zone_tmp.exists() {
        DirBuilder::new()
            .mode(0o1777)
            .create(&zone_tmp)
            .context(format!("unable to create zone tmp directory: {}", zonename))?;
    }

    Ok(())
}
