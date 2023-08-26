use clap::Parser;
use common::init_slog_logging;
use miette::{bail, miette, IntoDiagnostic, Result, WrapErr};
use opczone::{
    brand::Brand,
    build::{bundle::Bundle, run_action},
    get_zonepath_parent_ds,
    vmext::get_brand_config,
};
use solarm_utils::zfs::{
    clone as zfs_clone, create as zfs_create, CloneRequestBuilder, CreateRequestBuilder,
};
use std::{
    fs::DirBuilder,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'z')]
    zonename: String,

    #[arg(short = 'R')]
    zonepath: String,

    #[arg(short = 'q')]
    quota: i32,

    #[arg(long, default_value = "native")]
    brand: Brand,

    #[arg(short = 't')]
    image_uuid: Option<uuid::Uuid>,

    #[arg(short = 'b')]
    build_bundle: Option<PathBuf>,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let cli: Cli = Cli::parse();

    let _cfg = get_brand_config(&cli.zonename)?;

    if cli.image_uuid.is_some() && cli.build_bundle.is_some() {
        bail!("can only either deploy an image production by setting and image or build an image by setting build bundle. Both are set, bailing")
    }

    if cli.image_uuid.is_none() && cli.build_bundle.is_none() {
        bail!("must have image uuid or build bundle specified")
    }

    setup_dataset(
        &cli.zonename,
        &cli.zonepath,
        cli.quota,
        cli.image_uuid,
        cli.build_bundle.clone(),
        cli.brand.clone(),
    )?;

    setup_zone_fs(&cli.zonename, &cli.zonepath, cli.brand.clone())?;

    if let Some(build_bundle) = cli.build_bundle {
        let mut bundle = Bundle::new(&build_bundle).map_err(|err| miette!("{:?}", err))?;
        let bundle_audit = bundle.get_audit_info();

        //Install a base image by running the first IPS action in the GZ
        if bundle_audit.is_base_image() {
            //Run first IPS action to install base image
            if let Some(ips_action) = bundle.pop_action() {
                run_action(&cli.zonepath, &cli.zonename, &bundle, ips_action)?;
            }
        }

        //Save image bundle inside the image
        bundle.save_to_zone(&cli.zonepath)?;
    }

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
    brand: Brand,
) -> Result<()> {
    let parent_dataset = get_zonepath_parent_ds(zonepath)?;

    let zone_dataset_name = format!("{}/{}", parent_dataset, zonename);
    let root_dataset_name = format!("{}/{}", zone_dataset_name, "root");
    let vroot_dataset_name = format!("{}/{}", zone_dataset_name, "vroot");

    let quota_arg = format!("{}g", zonequota);

    // zoneadm already creates a dataset for the zone and does not swallow a -x argument
    // thus we do not need to create the top level dataset
    // dataset_create(&zone_dataset_name, false)?;

    if let Some(image) = image {
        let snapshot = format!("{}/{}@final", parent_dataset, image.to_string());
        let clone_request = CloneRequestBuilder::default()
            .snapshot(snapshot)
            .target(root_dataset_name)
            .add_property("devices", "off")
            .add_property("quota", &quota_arg)
            .build()
            .into_diagnostic()?;
        zfs_clone(&clone_request).into_diagnostic()?;
    } else if let Some(bundle_path) = build_bundle {
        let bundle = Bundle::new(&bundle_path).map_err(|err| miette!("{:?}", err))?;
        let audit_info = bundle.get_audit_info();
        if audit_info.is_base_image() {
            let create_root_request = CreateRequestBuilder::default()
                .name(&root_dataset_name)
                .add_property("devices", "off")
                .add_property("quota", &quota_arg)
                .build()
                .into_diagnostic()?;
            let create_vroot_request = CreateRequestBuilder::default()
                .name(&vroot_dataset_name)
                .add_property("mountpoint", "none")
                .build()
                .into_diagnostic()?;

            zfs_create(&create_root_request).into_diagnostic()?;
            zfs_create(&create_vroot_request).into_diagnostic()?;
        } else if let Some(image_name) = bundle.document.base_on {
            let image_uuid = opczone::image::find_image_by_name(&image_name)?
                .ok_or(miette!("no image found with name {}", &image_name))?;
            let root_snapshot = format!("{}/{}/root@final", parent_dataset, image_uuid.to_string());
            let vroot_snapshot =
                format!("{}/{}/vroot@final", parent_dataset, image_uuid.to_string());
            let root_clone_request = CloneRequestBuilder::default()
                .snapshot(root_snapshot)
                .target(root_dataset_name)
                .add_property("devices", "off")
                .add_property("quota", &quota_arg)
                .build()
                .into_diagnostic()?;

            let vroot_clone_request = CloneRequestBuilder::default()
                .snapshot(vroot_snapshot)
                .target(vroot_dataset_name)
                .add_property("devices", "off")
                .add_property("quota", &quota_arg)
                .add_property("mountpoint", "none")
                .add_property("canmount", "off")
                .build()
                .into_diagnostic()?;

            zfs_clone(&root_clone_request).into_diagnostic()?;
            zfs_clone(&vroot_clone_request).into_diagnostic()?;
        }
    } else if brand == Brand::Bhyve || brand == Brand::Propolis {
        println!("Empty VM creation not yet implemented");
        todo!()
    } else {
        bail!("neither image uuid or build bundle specified this would create an empty (unusable) zone")
    }

    Ok(())
}

fn setup_zone_fs(zonename: &str, zonepath: &str, brand: Brand) -> Result<()> {
    let config_path = Path::new(zonepath).join("config");
    let meta_path = Path::new(zonepath).join("meta");

    if !config_path.exists() {
        DirBuilder::new()
            .mode(0o755)
            .create(&config_path)
            .into_diagnostic()
            .wrap_err(format!(
                "unable to create zone config directory: {}",
                zonename
            ))?;
    }

    if brand == Brand::Image {
        if !meta_path.exists() {
            DirBuilder::new()
                .mode(0o755)
                .create(&meta_path)
                .into_diagnostic()
                .wrap_err(format!(
                    "unable to create zone config directory: {}",
                    zonename
                ))?;
        }
    }

    Ok(())
}
