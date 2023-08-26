use clap::{Parser, Subcommand, ValueEnum};
use miette::IntoDiagnostic;

#[derive(Debug, ValueEnum, Clone)]
pub enum NetTypeArg {
    V6Only,
    DualStack,
}

impl ToString for NetTypeArg {
    fn to_string(&self) -> String {
        match self {
            NetTypeArg::V6Only => String::from("v6-only"),
            NetTypeArg::DualStack => String::from("dual-stack"),
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Create {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
        name: String,
        #[arg(default_value_t = NetTypeArg::V6Only)]
        net: NetTypeArg,
        cidrs: Vec<String>,
        #[arg(long, short)]
        listen_ip: Option<String>,
        #[arg(long, short)]
        peer: Option<String>,
    },
    List {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
    },
    Install {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
        name: String,
        external_nic: String,
        image_uuid: uuid::Uuid,
    },
    //TODO: Uninstall command
    Get {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
        name: String,
    },
    Delete {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
        name: String,
    },
    GetNewAddress {
        #[arg(short, long)]
        tenant: Option<uuid::Uuid>,
        name: String,
    },
}

fn main() -> miette::Result<()> {
    let cli: Cli = Cli::parse();

    let cfg = config::open()?;

    let handler = vpcd::Handler::new(cfg)?;

    let action = match cli.command {
        Command::Create {
            tenant,
            name,
            net,
            cidrs,
            listen_ip,
            peer,
        } => {
            let backing: vpcd::VPCBacking = if let Some(ip) = listen_ip {
                if let Some(peer) = peer {
                    if let Some((peer_add_str, peer_port)) = peer.split_once(":") {
                        vpcd::VPCBacking::DirectOverlay {
                            listen_addr: ip.parse().into_diagnostic()?,
                            peer_addr: peer_add_str.parse().into_diagnostic()?,
                            peer_port: peer_port.parse().into_diagnostic()?,
                        }
                    } else {
                        miette::bail!(
                            "peer must contain an ip address port combination seperated by :"
                        )
                    }
                } else {
                    miette::bail!("both listen ip and peer are required to make an overlay")
                }
            } else {
                vpcd::VPCBacking::Etherstub
            };

            match net {
                NetTypeArg::V6Only => {
                    let net = if cidrs.len() > 0 {
                        Some(vpcd::NetTypeArg::V6Only(cidrs[0].clone()))
                    } else {
                        None
                    };
                    vpcd::VPCAction::Create {
                        tenant,
                        name,
                        net,
                        backing,
                    }
                }
                NetTypeArg::DualStack => {
                    let net = if cidrs.len() > 1 {
                        Some(vpcd::NetTypeArg::DualStack(
                            cidrs[0].clone(),
                            cidrs[1].clone(),
                        ))
                    } else if cidrs.len() == 1 {
                        return Err(miette::miette!(
                            "at least two ip cidrs are required for dualstack"
                        ));
                    } else {
                        None
                    };

                    vpcd::VPCAction::Create {
                        tenant,
                        name,
                        net,
                        backing,
                    }
                }
            }
        }
        Command::List { tenant } => vpcd::VPCAction::List { tenant },
        Command::Get { tenant, name } => vpcd::VPCAction::Get {
            tenant: tenant.unwrap_or(uuid::Builder::nil().into_uuid()),
            name,
        },
        Command::Delete { tenant, name } => vpcd::VPCAction::Delete {
            tenant: tenant.unwrap_or(uuid::Builder::nil().into_uuid()),
            name,
        },
        Command::GetNewAddress { tenant, name } => vpcd::VPCAction::GetNewAddress {
            tenant: tenant.unwrap_or(uuid::Builder::nil().into_uuid()),
            name,
        },
        Command::Install {
            tenant,
            name,
            external_nic,
            image_uuid,
        } => vpcd::VPCAction::Install {
            tenant,
            name,
            external_nic,
            image_uuid,
        },
    };

    let result = handler.handle(&action)?;

    match result {
        vpcd::VPCResponse::Empty => {}
        vpcd::VPCResponse::Address(addr) => {
            println!("Address: {:#?}", addr);
        }
        vpcd::VPCResponse::List(list) => {
            for net in list {
                println!("Network: {:#?}", net);
            }
        }
        vpcd::VPCResponse::One(net) => {
            println!("{:#?}", net);
        }
    }

    Ok(())
}
