use std::{fs::File, path::Path, process::Command};

use crate::machine::Payload;
use anyhow::{bail, Result};
use common::info;

/// Get the zones dataset name from the mounts
pub fn get_zone_dataset(zonepath: &str) -> Result<String> {

    for mnt in common::illumos::mounts()? {
        if &mnt.mount_point == zonepath {
            return Ok(mnt.special);
        }
    }

    bail!("No dataset found for mountpoint {}", zonepath)
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

    let vroot_ds_name_ending = format!("{}/vroot", zonename);

    for mnt in common::illumos::mounts()? {
        if mnt.special.ends_with(&vroot_ds_name_ending) {
            return Ok(mnt.special);
        }
    }

    bail!("Can not find vroot for zone {}", zonename)
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

    bail!("No dataset found for mountpoint {}", dname)
}

pub fn get_parent_dataset_path(zonepath: &str) -> Result<&str> {
    let (dname, _) = match common::path_split(zonepath) {
        Some(v) => v,
        None => bail!("could not split zonepath {}", zonepath),
    };
    Ok(dname)
}

pub fn get_config(zonename: &str, zonepath: &str) -> Result<Payload> {
    let (parent_ds_path, _) = match common::path_split(zonepath) {
        Some(v) => v,
        None => bail!("could not split zonepath {}", zonepath),
    };

    let config_path = Path::new(parent_ds_path)
        .join("config")
        .join(format!("{}.json", zonename));

    let cfg_file = File::open(config_path)?;

    let cfg = serde_json::from_reader(cfg_file)?;

    Ok(cfg)
}


pub fn dataset_create_with(dataset: &str, parents: bool, properties: &[(String,String)]) -> Result<()> {
    if dataset.contains('@') {
        bail!("no @ allowed here");
    }

    let properties: Vec<String> = properties.iter().map(|(key,value)| format!("{}={}", key, value)).collect();

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
        bail!("zfs create failed: {}", errmsg);
    }

    Ok(())
}