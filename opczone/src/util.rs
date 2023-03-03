use std::{fs::File, path::Path, process::Command};

use crate::machine::Payload;
use common::info;
use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum UtilError {
    #[error("No dataset found for mountpoint {0}")]
    NoDataSetFound(String),
    #[error("Can not find vroot for zone {0}")]
    NoVRootFound(String),
    #[error("cannot split zonepath {0}")]
    CannotSplitZonePath(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
    #[error("no snapshot allowed as target of this functioni: got {0}")]
    NoSnapShotNameAllowed(String),
    #[error("ZFS create failed: {0}")]
    ZFSCreateFailed(String),
    #[error(transparent)]
    IllumosError(#[from] common::illumos::IllumosError),
}

type Result<T> = miette::Result<T, UtilError>;

/// Get the zones dataset name from the mounts
pub fn get_zone_dataset(zonepath: &str) -> Result<String> {
    for mnt in common::illumos::mounts()? {
        if &mnt.mount_point == zonepath {
            return Ok(mnt.special);
        }
    }

    Err(UtilError::NoDataSetFound(zonepath.clone().into()))
}

/// Get the zones volume root(vroot) dataset name from mnttab(4)
/// The vroot dataset is a delegated dataset that is used as root dataset
/// for all volumes of the zone.
/// The vroot datasets root is not mounted by default and only serves as
/// a location to attach volumes to.
/// Since we should only ever have one vroot dataset per zone we simply return the first
/// occurance in mntttab(4)
/// if somebody has added more vroot datasets to the zone bad things will happen.
pub fn get_zone_vroot_dataset(zonename: &str) -> Result<String> {
    let vroot_ds_name_ending = format!("{}/root", zonename);

    for mnt in common::illumos::mounts()? {
        if mnt.special.ends_with(&vroot_ds_name_ending) {
            return Ok(mnt.special.replace("root", "vroot"));
        }
    }

    Err(UtilError::NoVRootFound(zonename.clone().into()))
}

/// Gets the parent dataset of the zones zonepath.
/// Usually this is the zones pool on smartos or
/// the zones dataset on other illumos distributions
pub fn get_zonepath_parent_ds(zonepath: &str) -> Result<String> {
    //+ ZONEPATH=/zones/oflnc
    //+ dname=/zones
    //+ bname=oflnc
    //+ nawk -v p=/zones '{if ($1 == p) print $3}'
    //+ mount
    //+ PDS_NAME=rpool/zones
    let dname = get_parent_dataset_path(zonepath)?;

    for mnt in common::illumos::mounts()? {
        if &mnt.mount_point == dname {
            return Ok(mnt.special);
        }
    }

    Err(UtilError::NoDataSetFound(dname.clone().into()))
}

pub fn get_parent_dataset_path(zonepath: &str) -> Result<&str> {
    let (dname, _) = common::path_split(zonepath)
        .ok_or(UtilError::CannotSplitZonePath(zonepath.clone().into()))?;

    Ok(dname)
}

pub fn get_config(zonename: &str, zonepath: &str) -> Result<Payload> {
    let (parent_ds_path, _) = common::path_split(zonepath)
        .ok_or(UtilError::CannotSplitZonePath(zonepath.clone().into()))?;
    let config_path = Path::new(parent_ds_path)
        .join("config")
        .join(format!("{}.json", zonename));

    let cfg_file = File::open(config_path)?;

    let cfg = serde_json::from_reader(cfg_file)?;

    Ok(cfg)
}

pub fn dataset_create_with(
    dataset: &str,
    parents: bool,
    properties: &[(String, String)],
) -> Result<()> {
    if dataset.contains('@') {
        return Err(UtilError::NoSnapShotNameAllowed(dataset.clone().into()));
    }

    let properties: Vec<String> = properties
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();

    info!("CREATE DATASET: {}", dataset);

    let mut cmd = Command::new("/sbin/zfs");
    cmd.env_clear();
    cmd.arg("create");
    if parents {
        cmd.arg("-p");
    }
    for prop in properties {
        cmd.arg("-o");
        cmd.arg(&prop);
    }
    cmd.arg(dataset);

    let zfs = cmd.output()?;

    if !zfs.status.success() {
        let errmsg = String::from_utf8_lossy(&zfs.stderr);
        return Err(UtilError::ZFSCreateFailed(errmsg.to_string()));
    }

    Ok(())
}
