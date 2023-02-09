use crate::{get_zone_dataset, get_zonepath_parent_ds};
use common::{debug, info};
use std::process::{Command, Stdio};
use std::{fs::File, path::Path};
use std::{thread, time};
use thiserror::Error;

const ZONEADM: &str = "/usr/sbin/zoneadm";
const ZFS: &str = "/usr/sbin/zfs";
const GZIP: &str = "/usr/bin/gzip";

pub struct ImageManifest {}

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("zone error: {0}")]
    ZoneError(#[from] zone::ZoneError),

    #[error("anyhow: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Zone is {0} in state {1} which can not be exported")]
    UnableToExport(String, String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Image export failed: {0}")]
    ImageExportFailed(String),

    #[error("Could not convert string to UTF-8")]
    UTF8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, ImageError>;

#[derive(Debug, Clone)]
pub enum ImageType {
    Dataset,
    OCI,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageType::Dataset => write!(f, "dataset"),
            ImageType::OCI => write!(f, "oci"),
        }
    }
}

pub fn convert_zone_to_image(zonename: &str) -> Result<uuid::Uuid> {
    // Make sure zone is shutdown
    let zone = crate::get_zone(zonename)?;
    match zone.state() {
        zone::State::Installed => {}
        zone::State::ShuttingDown => {
            info!("Zone is shutting down waiting a bit to settle");
            let sleep_time = time::Duration::from_millis(50);
            thread::sleep(sleep_time);
        }
        zone::State::Running => {
            info!("Shutting down zone {}", zonename);
            match crate::run(&[ZONEADM, "-z", zonename, "shutdown"], None) {
                Ok(_) => {}
                Err(_) => {
                    info!("Unable to shutdown zone ignoring init and halting zone");
                    crate::run(&[ZONEADM, "-z", zonename, "halt"], None)?;
                }
            }
        }
        s => {
            return Err(ImageError::UnableToExport(
                zonename.clone().to_string(),
                format!("{:?}", s),
            ));
        }
    }

    let zds = get_zone_dataset(&zone.path().to_string_lossy())?;
    let snap_name = format!("{}@final", &zds);
    info!("Snaphotting {}", &zds);
    crate::run(&[ZFS, "snap", "-r", &snap_name], None)?;

    let datasets = crate::run_capture_stdout(
        &[
            ZFS, "list", "-t", "snapshot", "-r", "-H", "-o", "name", &zds,
        ],
        None,
    )?;

    let image_uuid = uuid::Uuid::new_v4();

    let pds = get_zonepath_parent_ds(&zone.path().to_string_lossy())?;

    let image_base_ds = format!("{}/{}", pds, image_uuid.hyphenated().to_string());

    let mut image_datasets: Vec<String> = vec![];

    for ds in datasets.split_terminator("\n").collect::<Vec<&str>>() {
        let target_ds_name = ds.replace(&zds, &image_base_ds).replace("@final", "");
        image_datasets.insert(0, target_ds_name.clone());
        debug!("Cloning {} -> {}", ds, &target_ds_name);
        crate::run(&[ZFS, "clone", ds, &target_ds_name], None)?;
    }

    for ds in image_datasets {
        crate::run(&[ZFS, "promote", &ds], None)?;
    }

    Ok(image_uuid)
}

pub fn export_image_as_dataset_format<P: AsRef<Path>>(
    image_uuid: uuid::Uuid,
    output_dir: P,
) -> Result<()> {
    // zfs send dataset into output directory
    let image_path = format!("/zones/{}", image_uuid.as_hyphenated().to_string());

    let image_ds = get_zone_dataset(&image_path)?;
    let snap_name = format!("{}@final", &image_ds);

    let image_filename = format!("{}.zfs.gz", image_uuid.as_hyphenated().to_string());

    let file_path = output_dir.as_ref().join(&image_filename);

    let file = File::create(&file_path)?;

    info!(
        "Exporting zone to zfs image file {} with gzip compression",
        file_path.display()
    );

    let zfs_send = Command::new(ZFS)
        .arg("send")
        .arg("-R")
        .arg(&snap_name)
        .stdout(Stdio::piped())
        .spawn()?;

    let gzip = Command::new(GZIP)
        .stdin(Stdio::from(zfs_send.stdout.unwrap()))
        .stdout(file)
        .stderr(Stdio::piped())
        .spawn()?;

    let output = gzip.wait_with_output()?;

    if output.status.success() {
        info!("Sucess");
        Ok(())
    } else {
        Err(ImageError::ImageExportFailed(String::from_utf8(
            output.stderr,
        )?))
    }
}

#[allow(unused_variables)]
pub fn export_zone_as_oci_format<P: AsRef<Path>>(zone: zone::Zone, output_dir: P) -> Result<()> {
    todo!()
}
