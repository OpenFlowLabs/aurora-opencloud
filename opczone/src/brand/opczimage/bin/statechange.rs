use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt};

use anyhow::{bail, Context, Result};
use clap::{ArgEnum, Parser};
use common::{init_slog_logging, info, debug};
use opczone::{
    brand::{ZONE_CMD_BOOT, ZONE_CMD_HALT, ZONE_CMD_READY, ZONE_CMD_UNMOUNT, ZONE_STATE_DOWN, build_zonecontrol_gz_path, build_zonemeta_gz_path},
    dladm::{
        create_vnic, does_aggr_exist, does_etherstub_exist, does_phys_exist, show_one_vnic,
        show_vnic, CreateVNICArgs, CreateVNICProps, does_vnic_exist,
    },
    machine::{OnDiskPayload},
    vmext::{get_brand_config, write_brand_config},
};
use thiserror::Error;

const DEFAULT_MTU: i32 = 1500;

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
    mount_to_diagnose: Option<String>,
}

fn main() -> Result<()> {
    let _log_guard = init_slog_logging(false, true)?;

    let cli: Cli = Cli::parse();

    let mut cfg = get_brand_config(&cli.zonename)?;

    match cli.subcommand {
        StateSubCMD::Pre => {
            match cli.statecommand {
                ZONE_CMD_READY => {
                    //pre-ready
                    info!("Pre-ready");
                    setup_zone_helper_directories(&cli.zonename, &cli.zonepath)?;
                    cfg = setup_net(&cli.zonename, &cli.zonepath, &cfg)?;
                }
                ZONE_CMD_HALT => {
                    //pre-halt
                    info!("Pre-halt");
                    cleanup_net(&cli.zonename, &cli.zonepath, &cfg)?;
                }
                _ => {}
            }
        }
        StateSubCMD::Post => match cli.statecommand {
            ZONE_CMD_READY => {
                //post-ready
                info!("Post-ready");
                setup_fw(&cli.zonename, &cli.zonepath, &cfg)?;
            }
            ZONE_CMD_BOOT => {
                // post-boot
                // We can't set a rctl until we have a process in the zone to
                // grab
                //TODO: Check why the brand script needs to do this and leave it to zonecfg for now.
                setup_cpu_baseline(&cli.zonename, &cli.zonepath, &cfg)?;
            }
            ZONE_CMD_UNMOUNT => {
                // post-unmount
                // Zone halt is hung unmounting, try to recover
                if cli.currentstate == ZONE_STATE_DOWN {
                    cleanup_mount(&cli.zonename, &cli.zonepath, cli.mount_to_diagnose)?;
                }
            }
            _ => {}
        },
    }

    write_brand_config(&cfg)?;

    Ok(())
}

fn setup_net(zonename: &str, _zonepath: &str, cfg: &OnDiskPayload) -> Result<OnDiskPayload> {
    let mut new_payload = cfg.clone();
    for (idx, mut nic) in cfg.nics.clone().into_iter().enumerate() {
        /*
        #
        # The nic tag for a device can come in
        # one of a few forms. It may:
        #
        # 1) Be a traditional tag which refers to a physical device or
        #    aggregation to create a VNIC over. The source of this
        #    mapping is dladm show-phys.
        #
        # 2) It can be the name of an etherstub. The source of these is
        #    from dladm show-etherstub
        #
        # Note: Overlay is not yet supported in OI as of 2022-08-12
        # TODO: Support for olverlay once it lands in dladm
        # 3) It can take the form of an overlay device rule. An overlay
        #    device rule is an invalid DLPI device and invalid nic tag.
        #    It has the form of <name>/<number>. For example,
        #    sdc_sdn/23. That refers to the overlay rule sdc_sdn. If we
        #    have an overlay rule, we may need to dynamically create the
        #    overlay device if it doesn't exist.
        #
        # To handle these cases, we first check if it's an overlay
        # device, and then if not, check the other cases.
        #
        */
        debug!("Checking for backing interface {}", &nic.nic_tag);
        if !does_phys_exist(&nic.nic_tag)
            && !does_aggr_exist(&nic.nic_tag)
            && !does_etherstub_exist(&nic.nic_tag)
        {
            //TODO: Overlay handling here
            bail!(StateChangeError::NoBackingInterface(nic.nic_tag))
        }

        debug!("Checking if VNIC {} already exists", &nic.interface);
        if !does_vnic_exist(&nic.interface) {
            debug!("Building dladm command");
            // Build the vnic args
            let mut nic_args: Vec<CreateVNICArgs> = vec![
                CreateVNICArgs::Temporary,
                CreateVNICArgs::Link(nic.nic_tag.clone()),
            ];
            let mut nic_opts: Vec<CreateVNICProps> = vec![
                CreateVNICProps::Zone(zonename.clone().to_owned()),
                CreateVNICProps::Mtu(DEFAULT_MTU),
            ];

            if let Some(vrid) = nic.vrrp_vrid {
                nic_args.push(CreateVNICArgs::Vrrp(vrid));
            }

            if let Some(mac_addr) = nic.mac.clone() {
                nic_args.push(CreateVNICArgs::Mac(mac_addr));
            }

            if let Some(vlan_id) = nic.vlan_id {
                nic_args.push(CreateVNICArgs::Vlan(vlan_id));
            }

            create_vnic(&nic.interface, Some(nic_args), Some(nic_opts))?;
            debug!("VNIC created");
            //If mac address is empty get it from the newly created nic and save it into the config
            if nic.mac.is_none() {
                let info = show_one_vnic(&nic.interface)?;
                nic.mac = Some(info.mac)
            }

            //TODO: Setup protection linkprop

            //TODO: set allowed-ips

            //TODO: set dynamic-methods (needs upstream from illumos-joyent first)

            //TODO: set allowed-dhcp-cids

            //TODO: set promisc-filtered

            //TODO: Setup flowadm (block ports that should not go to the outside world)
            /*
                mac_addr=`dladm show-vnic -p \
                        -o MACADDRESS $nic | tr ':' '_'`
                MACADDR that has : replaced with _ becomes flowname
                flowadm add-flow -t -l $nic \
                        -a transport=tcp,remote_port=$port \
                        -p maxbw=0 f${mac_addr}_br_${port}
            */

            //TODO: Setup vnd device once illumos-gate gets support for that

            new_payload.nics[idx] = nic;
        }
    }
    Ok(new_payload)
}

#[derive(Error, Debug, Clone)]
enum StateChangeError {
    #[error("no backing nic found with tag {0}")]
    NoBackingInterface(String),
}

fn setup_fw(zonename: &str, _zonepath: &str, cfg: &OnDiskPayload) -> Result<()> {
    Ok(())
}

fn setup_cpu_baseline(zonename: &str, _zonepath: &str, cfg: &OnDiskPayload) -> Result<()> {
    Ok(())
}

fn cleanup_net(zonename: &str, _zonepath: &str, cfg: &OnDiskPayload) -> Result<()> {
    Ok(())
}

fn cleanup_mount(zonename: &str, _zonepath: &str, mount_to_diagnose: Option<String>) -> Result<()> {
    Ok(())
}

// This function runs in the global zone to make sure the directories the zone will need are setup
fn setup_zone_helper_directories(zonename: &str, _zonepath: &str) -> Result<()> {
    let zonecontrol_path = build_zonecontrol_gz_path(zonename);
    //mkdir -m755 -p /var/zonecontrol/${ZONENAME}
    if !zonecontrol_path.exists() {
        DirBuilder::new()
            .mode(0o755)
            .recursive(true)
            .create(&zonecontrol_path)
            .context(format!(
                "unable to create zone control directory {}",
                zonename
            ))?;
    }

    let zonemeta_path = build_zonemeta_gz_path(zonename);
    //mkdir -m755 -p /var/zonemeta/${ZONENAME}
    if !zonemeta_path.exists() {
        DirBuilder::new()
            .mode(0o755)
            .recursive(true)
            .create(&zonemeta_path)
            .context(format!(
                "unable to create zone meta directory {}",
                zonename
            ))?;
    }

    Ok(())
}
