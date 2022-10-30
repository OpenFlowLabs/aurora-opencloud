use std::{path::{PathBuf, Path}, str::FromStr};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// States
pub const ZONE_STATE_CONFIGURED: i32 = 0; //(never see)
pub const ZONE_STATE_INCOMPLETE: i32 = 1; //(never see)
pub const ZONE_STATE_INSTALLED: i32 = 2;
pub const ZONE_STATE_READY: i32 = 3;
pub const ZONE_STATE_RUNNING: i32 = 4;
pub const ZONE_STATE_SHUTTING_DOWN: i32 = 5;
pub const ZONE_STATE_DOWN: i32 = 6;
pub const ZONE_STATE_MOUNTED: i32 = 7;

// cmd
pub const ZONE_CMD_READY: i32 = 0;
pub const ZONE_CMD_BOOT: i32 = 1;
pub const ZONE_CMD_FORCEBOOT: i32 = 2;
pub const ZONE_CMD_REBOOT: i32 = 3;
pub const ZONE_CMD_HALT: i32 = 4;
pub const ZONE_CMD_UNINSTALLING: i32 = 5;
pub const ZONE_CMD_MOUNT: i32 = 6;
pub const ZONE_CMD_FORCEMOUNT: i32 = 7;
pub const ZONE_CMD_UNMOUNT: i32 = 8;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ZoneSubProcExitCode(i32);

impl ZoneSubProcExitCode {
    pub const ZONE_SUBPROC_OK: ZoneSubProcExitCode = ZoneSubProcExitCode(0);
    pub const ZONE_SUBPROC_USAGE: ZoneSubProcExitCode = ZoneSubProcExitCode(253);
    pub const ZONE_SUBPROC_NOTCOMPLETE: ZoneSubProcExitCode = ZoneSubProcExitCode(254);
    pub const ZONE_SUBPROC_FATAL: ZoneSubProcExitCode = ZoneSubProcExitCode(255);
}

pub const ZONECONTROL_GZ_PATH: &str = "/var/zonecontrol";
pub const ZONECONTROL_NGZ_PATH: &str = "/.zonecontrol";
pub const ZONEMETA_GZ_PATH: &str = "/var/zonemeta";
pub const ZONEMETA_NGZ_PATH: &str = "/.zonemeta";

pub fn build_zonecontrol_gz_path(zonename: &str) -> PathBuf {
    Path::new(ZONECONTROL_GZ_PATH).join(zonename)
}

pub fn build_zonemeta_gz_path(zonename: &str) -> PathBuf {
    Path::new(ZONEMETA_GZ_PATH).join(zonename)
}

#[derive(Debug, Error)]
pub enum BrandError{
    #[error("brand {0} is unknown")]
    NotKnown(String)
}


#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Brand {
    Bhyve,
    Image,
    Native,
    Propolis,
}

impl FromStr for Brand {
    type Err = BrandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bhyve" => Ok(Brand::Bhyve),
            "image" => Ok(Brand::Image),
            "native" => Ok(Brand::Native),
            "propolis" => Ok(Brand::Propolis),
            x => Err(BrandError::NotKnown(x.clone().to_string()))
        }
    }
}

impl std::fmt::Display for Brand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Brand::Bhyve => write!(f, "opcbhyve"),
            Brand::Image => write!(f, "opczimage"),
            Brand::Native => write!(f, "opcnative"),
            Brand::Propolis => write!(f, "opcpropolis"),
        }
    }
}
