use bonsaidb::{
    core::schema::{InsertError, SerializedCollection},
    local::config::{Builder, StorageConfiguration},
};
use miette::Diagnostic;
use std::{net::IpAddr, process::Command};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    Insert(#[from] InsertError<vpc::VPC>),

    #[error(transparent)]
    VPC(#[from] vpc::Error),

    #[error(transparent)]
    BonsaiDB(#[from] bonsaidb::core::Error),

    #[error(transparent)]
    BonsaiDBLocal(#[from] bonsaidb::local::Error),

    #[error("duplicate vpc's detected only expected one with the same name per tenant")]
    DuplicateVPCS,

    #[error("one vpc should be in the answer but none found")]
    NoVPC,

    #[error("dladm command failed with non zero exit")]
    DladmNonZeroExit,

    #[error("the listen ip for VXLAN must be either mentioned in the config or as argument to this command")]
    NoListenIP,
}

type Result<T> = miette::Result<T, Error>;

#[derive(Debug)]
pub enum NetTypeArg {
    V6Only(String),
    DualStack(String, String),
}

#[derive(Debug)]
pub enum VPCAction {
    Create {
        tenant: Option<uuid::Uuid>,
        name: String,
        net: Option<NetTypeArg>,
        backing: VPCBacking,
    },
    List {
        tenant: Option<uuid::Uuid>,
    },
    Get {
        tenant: uuid::Uuid,
        name: String,
    },
    Delete {
        tenant: uuid::Uuid,
        name: String,
    },
    GetNewAddress {
        tenant: uuid::Uuid,
        name: String,
    },
}

#[derive(Debug)]
pub enum VPCBacking {
    Etherstub,
    DirectOverlay {
        listen_addr: IpAddr,
        peer_addr: IpAddr,
        peer_port: i32,
    },
}

impl Default for VPCBacking {
    fn default() -> Self {
        Self::Etherstub
    }
}

#[derive(Debug)]
pub enum VPCResponse {
    Empty,
    Address(vpc::Addr),
    List(Vec<vpc::VPC>),
    One(vpc::VPC),
}

pub enum Handler {
    Local {
        config: config::Config,
        database: bonsaidb::local::Database,
    },
    Remote,
}

impl Handler {
    pub fn new(config: config::Config) -> Result<Self> {
        let storage = StorageConfiguration::new(&config.vpc_db_path);
        Ok(Self::Local {
            config,
            database: bonsaidb::local::Database::open::<vpc::VPC>(storage)?,
        })
    }

    pub fn handle(&self, action: &VPCAction) -> Result<VPCResponse> {
        match self {
            Handler::Local { database, config } => handle_local(config, &database, action),
            Handler::Remote => handle_remote(action),
        }
    }
}

fn handle_local(
    _config: &config::Config,
    db: &bonsaidb::local::Database,
    action: &VPCAction,
) -> Result<VPCResponse> {
    match action {
        VPCAction::Create {
            tenant,
            name,
            net,
            backing,
        } => {
            let mut builder =
                vpc::Builder::new(&tenant.unwrap_or(uuid::Builder::nil().into_uuid()), name);

            if let Some(net) = net {
                match net {
                    NetTypeArg::V6Only(v6_string) => {
                        builder.set_net(vpc::Net::net_v6only_from_string(v6_string)?);
                    }
                    NetTypeArg::DualStack(v4_string, v6_string) => {
                        builder.set_net(vpc::Net::net_dualstack_from_string(v4_string, v6_string)?);
                    }
                }
            }

            let vnet = builder.into_vpc();

            let vnet_doc = vnet.push_into(db)?;

            match backing {
                VPCBacking::Etherstub => {
                    let dladm_out = Command::new("dladm")
                        .arg("create-etherstub")
                        .arg(format!("{}{}", name, vnet_doc.header.id))
                        .output()
                        .or_else(|v| {
                            vnet_doc.delete(db).unwrap();
                            Err(v)
                        })?;

                    if !dladm_out.status.success() {
                        vnet_doc.delete(db)?;
                        println!("{}", String::from_utf8_lossy(&dladm_out.stderr));
                        Err(Error::DladmNonZeroExit)
                    } else {
                        Ok(VPCResponse::Empty)
                    }
                }
                VPCBacking::DirectOverlay {
                    listen_addr,
                    peer_addr,
                    peer_port,
                } => {
                    let dladm_out = Command::new("dladm")
                        .arg("create-overlay")
                        .arg("-e")
                        .arg("vxlan")
                        .arg("-s")
                        .arg("files")
                        .arg("-v")
                        .arg(vnet_doc.header.id.to_string())
                        .arg("-p")
                        .arg(format!("vxlan/listen_ip={}", listen_addr.to_string()))
                        .arg("-p")
                        .arg(format!("vxlan/listen_port={}", 4789 + vnet_doc.header.id))
                        .arg("-p")
                        .arg(format!("direct/dest_ip={}", peer_addr.to_string()))
                        .arg("-p")
                        .arg(format!("direct/dest_port={}", peer_port))
                        .arg(format!("{}{}", name, vnet_doc.header.id))
                        .output()
                        .or_else(|v| {
                            vnet_doc.delete(db).unwrap();
                            Err(v)
                        })?;

                    if !dladm_out.status.success() {
                        vnet_doc.delete(db)?;
                        println!("{}", String::from_utf8_lossy(&dladm_out.stderr));
                        Err(Error::DladmNonZeroExit)
                    } else {
                        Ok(VPCResponse::Empty)
                    }
                }
            }
        }
        VPCAction::List { tenant } => Ok(VPCResponse::List(
            vpc::VPC::all(db)
                .query()?
                .into_iter()
                .filter_map(|doc| {
                    if let Some(tenant) = tenant {
                        if &doc.contents.tenant_id == tenant {
                            Some(doc.contents)
                        } else {
                            None
                        }
                    } else {
                        Some(doc.contents)
                    }
                })
                .collect(),
        )),
        VPCAction::Get { tenant, name } => {
            let resuting_list: Vec<vpc::VPC> = vpc::VPC::all(db)
                .query()?
                .into_iter()
                .filter_map(|doc| {
                    if (&doc.contents.tenant_id == tenant) && (&doc.contents.name == name) {
                        Some(doc.contents)
                    } else {
                        None
                    }
                })
                .collect();
            if resuting_list.len() != 1 {
                Err(Error::DuplicateVPCS)
            } else {
                Ok(VPCResponse::One(resuting_list[0].clone()))
            }
        }
        VPCAction::Delete { tenant, name } => {
            let resuting_list: Vec<_> = vpc::VPC::all(db)
                .query()?
                .into_iter()
                .filter(|doc| {
                    if (&doc.contents.tenant_id == tenant) && (&doc.contents.name == name) {
                        true
                    } else {
                        false
                    }
                })
                .collect();

            if resuting_list.len() != 1 {
                return Err(Error::DuplicateVPCS);
            }

            let doc = resuting_list.first().ok_or(Error::NoVPC)?;

            let dladm_out = Command::new("dladm")
                .arg("delete-etherstub")
                .arg(format!("{}{}", name, doc.header.id))
                .output()
                .or_else(|v| {
                    doc.delete(db).unwrap();
                    Err(v)
                })?;

            if !dladm_out.status.success() {
                println!("{}", String::from_utf8_lossy(&dladm_out.stderr));
                Err(Error::DladmNonZeroExit)
            } else {
                doc.delete(db)?;
                Ok(VPCResponse::Empty)
            }
        }
        VPCAction::GetNewAddress { tenant, name } => {
            let mut resuting_list: Vec<_> = vpc::VPC::all(db)
                .query()?
                .into_iter()
                .filter(|doc| {
                    if (&doc.contents.tenant_id == tenant) && (&doc.contents.name == name) {
                        true
                    } else {
                        false
                    }
                })
                .collect();
            let document = &mut resuting_list[0];

            let addr = document.contents.reserve_new_address()?;

            document.update(db)?;

            Ok(VPCResponse::Address(addr))
        }
    }
}

fn handle_remote(_action: &VPCAction) -> Result<VPCResponse> {
    todo!()
}
