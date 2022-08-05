
use std::{path::Path, fs::DirBuilder, os::unix::fs::DirBuilderExt};

use common::{init_slog_logging};
use anyhow::{Result, Context};
use clap::{Parser, ArgEnum};
use opczone::brand::{ZONE_CMD_READY, ZONE_CMD_HALT, ZONE_CMD_BOOT, ZONE_CMD_UNMOUNT};

#[derive(ArgEnum, Debug, Clone)] // ArgEnum here
#[clap(rename_all = "kebab_case")]
enum StateSubCMD {
    Pre,
    Post,
}

#[derive(Parser)]
struct Cli {
    #[clap(value_parser, arg_enum)]
    subcommand: StateSubCMD,

    #[clap(value_parser)]
    zonename: String,

    #[clap(value_parser)]
    zonepath: String,

    #[clap(value_parser)]
    currentstate: i32,

    #[clap(value_parser)]
    statecommand: i32,

    #[clap(value_parser)]
    altroot: Option<String>,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false)?;

    let cli: Cli = Cli::parse();

    match cli.subcommand {
        StateSubCMD::Pre => {
            match cli.statecommand {
                ZONE_CMD_READY => { //pre-ready
                    setup_zone_control_dir(&cli.zonename, &cli.zonepath)?;
                },
                ZONE_CMD_HALT => { //pre-halt
                    
                },
                _ => {}
            }
        },
        StateSubCMD::Post => match cli.statecommand {
            ZONE_CMD_READY => {}
            ZONE_CMD_BOOT => {}
            ZONE_CMD_UNMOUNT => {}
            _ => {}
        },
    }

    Ok(())
}

// This function runs in the global zone to make sure the directories the zone will need are setup
fn setup_zone_control_dir(zonename: &str, _zonepath: &str) -> Result<()> {
    let zonecontrol_path = Path::new("/var/zonecontrol").join(zonename);
    //mkdir -m755 -p /var/zonecontrol/${ZONENAME}
    if !zonecontrol_path.exists() {
        DirBuilder::new().mode(0o755).create(&zonecontrol_path)
        .context(format!("unable to create zone control directory {}", zonename))?;
    }

    Ok(())
}
