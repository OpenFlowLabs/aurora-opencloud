use crate::get_zone_dataset;
use common::info;
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

pub fn convert_zone_to_image<P: AsRef<Path>>(
    zonename: &str,
    output_dir: P,
    image_type: ImageType,
) -> Result<()> {
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
            crate::run(&[ZONEADM, "-z", zonename, "shutdown"], None)?;
        }
        s => {
            return Err(ImageError::UnableToExport(
                zonename.clone().to_string(),
                format!("{:?}", s),
            ));
        }
    }

    // if it is dataset type image
    match image_type {
        ImageType::Dataset => {
            export_zone_as_dataset_format(zone, output_dir)?;
        }
        ImageType::OCI => {
            // run oci export and write oci compliant image files in output directory
            export_zone_as_oci_format(zone, output_dir)?;
        }
    }

    Ok(())
}

fn export_zone_as_dataset_format<P: AsRef<Path>>(zone: zone::Zone, output_dir: P) -> Result<()> {
    // snapshot zone dataset recursive
    // zfs send dataset into output directory
    let zds = get_zone_dataset(&zone.path().to_string_lossy())?;
    let snap_name = format!("{}@final", &zds);
    info!("Snaphotting {}", &zds);
    crate::run(&[ZFS, "snap", "-r", &snap_name], None)?;

    let file_name = output_dir.as_ref().join("image_zfs.gz");

    let file = File::create(file_name.clone())?;

    info!(
        "Exporting zone to zfs image file {} with gzip compression",
        file_name.display()
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
fn export_zone_as_oci_format<P: AsRef<Path>>(zone: zone::Zone, output_dir: P) -> Result<()> {
    todo!()
}
