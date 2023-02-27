use crate::{run, run_capture_stdout};
use common::debug;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum DladmError {
    #[error(transparent)]
    OPCError(#[from] crate::OPCZoneError),
}

type Result<T> = miette::Result<T, DladmError>;

const DLADM_BIN: &str = "/usr/sbin/dladm";

pub enum LinkKind {
    Phys,
    Aggr,
    Etherstub,
    Vnic,
    Bridge,
}

pub struct LinkInfo {
    pub class: LinkKind,
    pub name: String,
    //pub mtu: String,
    //pub state: String,
}

#[derive(Debug, Clone, Default)]
pub struct VnicInfo {
    pub name: String,
    pub over: String,
    pub mac: String,
    pub mac_type: String,
}

pub fn show_phys() -> Result<Vec<LinkInfo>> {
    // dladm show-phys -p -o LINK
    show("phys")
}

pub fn does_phys_exist(name: &str) -> bool {
    does_exist("phys", name)
}

pub fn show_aggr() -> Result<Vec<LinkInfo>> {
    // dladm show-aggr -p -o LINK
    show("aggr")
}

pub fn does_aggr_exist(name: &str) -> bool {
    does_exist("aggr", name)
}

pub fn show_etherstub() -> Result<Vec<LinkInfo>> {
    // dladm show-etherstub -p -o LINK
    show("etherstub")
}

pub fn does_etherstub_exist(name: &str) -> bool {
    does_exist("etherstub", name)
}

pub fn show_vnic() -> Result<Vec<VnicInfo>> {
    let dladm_args = [
        DLADM_BIN,
        "show-vnic",
        "-p",
        "-o",
        "LINK,OVER,MACADDRESS,MACADDRTYPE",
    ];
    let stdout = run_capture_stdout(&dladm_args, None)?;
    Ok(stdout
        .lines()
        .map(|l| {
            let line = l.clone().to_owned().replace("\\:", ";");
            let line: Vec<&str> = line.split(":").collect();
            if line.len() < 4 {
                panic!("not enough output from dladm show-phys line: got {}", l);
            }
            VnicInfo {
                name: line[0].to_owned(),
                over: line[1].to_owned(),
                mac: line[2].to_owned().replace(";", ":"),
                mac_type: line[3].to_owned(),
            }
        })
        .collect())
}

pub fn show_one_vnic(name: &str) -> Result<VnicInfo> {
    let dladm_args = [
        DLADM_BIN,
        "show-vnic",
        "-p",
        "-o",
        "LINK,OVER,MACADDRESS,MACADDRTYPE",
        name,
    ];
    let stdout = run_capture_stdout(&dladm_args, None)?;
    let infos: Vec<VnicInfo> = stdout
        .lines()
        .map(|l| {
            let line = l.clone().to_owned().replace("\\:", ";");
            let line: Vec<&str> = line.split(":").collect();
            if line.len() < 4 {
                panic!("not enough output from dladm show-phys line: got {}", l);
            }
            VnicInfo {
                name: line[0].to_owned(),
                over: line[1].to_owned(),
                mac: line[2].to_owned().replace(";", ":"),
                mac_type: line[3].to_owned(),
            }
        })
        .collect();
    Ok(infos[0].clone())
}

pub fn does_vnic_exist(name: &str) -> bool {
    does_exist("vnic", name)
}

pub fn show_bridge() -> Result<Vec<LinkInfo>> {
    // dladm show-bridge -p -o LINK
    show("bridge")
}

pub fn does_bridge_exist(name: &str) -> bool {
    does_exist("bridge", name)
}

fn does_exist(kind: &str, name: &str) -> bool {
    let link_list = match show(kind) {
        Ok(v) => v,
        Err(_) => vec![],
    };
    for link in link_list {
        if &link.name == name {
            return true;
        }
    }
    false
}

fn show(class: &str) -> Result<Vec<LinkInfo>> {
    let dladm_args = [
        DLADM_BIN,
        &format!("show-{}", class),
        "-p",
        "-o",
        "LINK", //TODO: Figure out why MTU does not work on illumos but exists on SmartOS
    ];
    let stdout = run_capture_stdout(&dladm_args, None)?;
    Ok(stdout
        .lines()
        .map(|l| {
            let class = match class {
                "phys" => LinkKind::Phys,
                "aggr" => LinkKind::Aggr,
                "etherstub" => LinkKind::Etherstub,
                "vnic" => LinkKind::Vnic,
                "bridge" => LinkKind::Bridge,
                x => panic!(
                    "{} is not wired up correctly in show function, programmer error",
                    x
                ),
            };
            let line: Vec<&str> = l.split(":").collect();
            if line.len() < 1 {
                panic!("not enough output from dladm show-phys line: got {}", l);
            }
            LinkInfo {
                class: class,
                name: line[0].to_owned(),
            }
        })
        .collect())
}

#[derive(Debug)]
pub enum CreateVNICArgs {
    Vrrp(u8),
    Mac(String),
    Vlan(i32),
    Temporary,
    Link(String),
}

#[derive(Debug)]
pub enum CreateVNICProps {
    Mtu(i32),
    Zone(String),
}

pub fn create_vnic(
    name: &str,
    args: Option<Vec<CreateVNICArgs>>,
    opts: Option<Vec<CreateVNICProps>>,
) -> Result<()> {
    debug!(
        "Calling dladm with args: {:#?} and options {:#?}",
        &args, &opts
    );
    let mut dladm_args: Vec<String> = vec![DLADM_BIN.to_owned(), "create-vnic".to_owned()];
    if let Some(opts) = opts {
        dladm_args.push("-p".to_owned());
        let mut props: Vec<String> = vec![];
        for opt in opts {
            props.push(match opt {
                CreateVNICProps::Mtu(mtu) => format!("mtu={}", mtu),
                CreateVNICProps::Zone(zone) => format!("zone={}", zone),
            });
        }
        let prop_string = props.join(",");
        dladm_args.push(prop_string);
    }
    if let Some(args) = args {
        let mut has_vrrp = false;
        for arg in args {
            match arg {
                CreateVNICArgs::Vrrp(vrid) => {
                    has_vrrp = true;
                    dladm_args.push("-V".to_owned());
                    dladm_args.push(format!("{}", vrid));
                    dladm_args.push("-A".to_owned());
                    dladm_args.push("inet".to_owned());
                    dladm_args.push("-m".to_owned());
                    dladm_args.push("vrrp".to_owned());
                }
                CreateVNICArgs::Mac(mac) => {
                    if !has_vrrp {
                        dladm_args.push("-m".to_owned());
                        dladm_args.push(mac);
                    }
                }
                CreateVNICArgs::Vlan(vlan) => {
                    dladm_args.push("-v".to_owned());
                    dladm_args.push(format!("{}", vlan));
                }
                CreateVNICArgs::Temporary => {
                    dladm_args.push("-t".to_owned());
                }
                CreateVNICArgs::Link(link) => {
                    dladm_args.push("-l".to_owned());
                    dladm_args.push(link);
                }
            }
        }
    }

    dladm_args.push(name.to_owned());

    Ok(run(dladm_args.as_slice(), None)?)
}
