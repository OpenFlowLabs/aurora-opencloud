use std::{fs::File, path::Path};

use crate::machine::Payload;
use anyhow::{bail, Result};

/// Get the zones dataset name from the mounts
pub fn get_zone_dataset(zonepath: &str) -> Result<String> {

    for mnt in common::illumos::mounts()? {
        if &mnt.mount_point == zonepath {
            return Ok(mnt.special);
        }
    }

    bail!("No dataset found for mountpoint {}", zonepath)
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
