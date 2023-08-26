use bonsaidb::{
    core::schema::{InsertError, SerializedCollection},
    local::config::{Builder, StorageConfiguration},
};
use illumos_image_builder::dataset_clone;
use miette::Diagnostic;
use opczone::{brand::Brand, machine::VMAPIError};
use serde::Serialize;
use std::{
    fs::{DirBuilder, File},
    io::Write,
    net::IpAddr,
    path::Path,
    process::Command,
    string::FromUtf8Error,
};
use thiserror::Error;
use zone::ZoneError;

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

    #[error(transparent)]
    VMAPIError(#[from] VMAPIError),

    #[error(transparent)]
    ZoneError(#[from] ZoneError),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    UtilError(#[from] opczone::UtilError),

    #[error(transparent)]
    OPCZoneError(#[from] opczone::OPCZoneError),

    #[error("duplicate vpc's detected only expected one with the same name per tenant")]
    DuplicateVPCS,

    #[error("one vpc should be in the answer but none found")]
    NoVPC,

    #[error("dladm command failed with non zero exit")]
    DladmNonZeroExit,

    #[error("the listen ip for VXLAN must be either mentioned in the config or as argument to this command")]
    NoListenIP,

    #[error("mkisofs failed: {0}")]
    MkisofsError(String),

    #[error("dataset could not be cloned {0}")]
    DatasetCloneError(String),
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
    Install {
        tenant: Option<uuid::Uuid>,
        name: String,
        external_nic: String,
        image_uuid: uuid::Uuid,
    },
    Uninstall {
        tenant: Option<uuid::Uuid>,
        name: String,
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
                        .arg("direct")
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
        VPCAction::Get { tenant, name } => get_vpc(db, tenant, name).map(|v| VPCResponse::One(v.1)),
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
        VPCAction::Install {
            tenant,
            name,
            external_nic,
            image_uuid,
        } => {
            //Retrieve VPC from DB
            let tenant = tenant.unwrap_or(uuid::Builder::nil().into_uuid());
            let vpc = get_vpc(db, &tenant, &name)?;
            let vswitch_name = get_vswitch_name(vpc.0, &vpc.1.name);

            let iso_root_name = "/vm/iso";
            let iso_path_root = Path::new(iso_root_name);

            if !iso_path_root.exists() {
                DirBuilder::new().recursive(true).create(&iso_path_root)?;
            }

            // Commands used manually are
            // VNIC Setup
            // dladm create-vnic -l <external_nic> [vnic_name]
            // dladm create-vnic -l <internal_net> [vnic_name]
            let external_vnic = setup_vnic_for_router(&external_nic, &vpc.1.name, true)?;
            let internal_vnic = setup_vnic_for_router(&vswitch_name, &vpc.1.name, false)?;

            // Zonecfg -z [vm_name] create -b << {Config}
            let mut create_payload = opczone::machine::CreatePayload::default();
            create_payload.brand = Brand::NativeBhyve;
            create_payload.image_uuid = Some(image_uuid.clone());
            let ram = 1024;
            let vcpus = 1;
            create_payload.ram = ram;
            create_payload.vcpus = vcpus;

            let vm_config = opczone::machine::define_vm(create_payload)?;
            let vm_uuid = vm_config.uuid.hyphenated().to_string();
            let mut zone_adm_handle = zone::Adm::new(&vm_uuid);
            let mut zone_cfg_handle = zone::Config::new(&vm_uuid);
            let seed_iso_path = iso_path_root.join(&vm_uuid).with_extension("iso");
            let z = opczone::get_zone(&vm_uuid)?;
            let parent_ds_name =
                opczone::get_zonepath_parent_ds(z.path().to_string_lossy().to_string().as_str())?;

            let disk_volume_path = format!("{}/{}/root_disk", &parent_ds_name, &vm_uuid);

            zone_cfg_handle
                .add_fs(&zone::Fs::default())
                .set_ty("lofs")
                .set_dir(iso_root_name)
                .set_special(iso_root_name)
                .set_options([String::from("ro"), String::from("nodevices")]);
            zone_cfg_handle
                .add_device(&zone::Device::default())
                .set_name(format!("/dev/zvol/rdsk/{}", &disk_volume_path));
            zone_cfg_handle.add_attr(&zone::Attr {
                name: String::from("bootdisk"),
                value: zone::AttributeValue::String(disk_volume_path.clone()),
            });
            zone_cfg_handle.add_attr(&zone::Attr {
                name: String::from("ram"),
                value: zone::AttributeValue::String(ram.to_string()),
            });
            zone_cfg_handle.add_attr(&zone::Attr {
                name: String::from("vcpus"),
                value: zone::AttributeValue::String(vcpus.to_string()),
            });
            zone_cfg_handle.add_attr(&zone::Attr {
                name: String::from("cdrom"),
                value: zone::AttributeValue::String(seed_iso_path.to_string_lossy().to_string()),
            });

            zone_cfg_handle.run_blocking()?;

            // zfs clone root_disk_vol_name vm_vol_name
            let src_name = format!("/zones/{}", &image_uuid.as_hyphenated().to_string());
            dataset_clone(&src_name, &disk_volume_path, None)
                .map_err(|err| Error::DatasetCloneError(err.to_string()))?;

            // Zoneadm install -z [vm_name]
            zone_adm_handle.install_blocking(vec![].as_slice())?;

            // Create cloud-init config under /tmp
            let t = tempfile::tempdir()?;

            File::create(t.path().join("meta-data"))?;

            let network_config_content = r#"version: 2
ethernets:
  eth0:
    dhcp4: false
    dhcp6: false"#;
            let mut network_config_file = File::create(t.path().join("network-config"))?;
            network_config_file.write_all(network_config_content.as_bytes())?;

            let user_data = UserData::new()
                .set(&["system", "host-name", "'vyos-blub'"])
                .to_string();
            let mut user_data_file = File::create(t.path().join("user-data"))?;
            user_data_file.write_all(user_data.as_bytes())?;

            // mkisofs -graft-points -dlrDJN -relaxed-filenames -o [ISO_PATH] -V cidata [user-data] [meta-data] [network-config]
            let mkisofs_out = Command::new("mkisofs")
                .arg("-graft-points")
                .arg("-dlrDJN")
                .arg("-relaxed-filenames")
                .arg("-o")
                .arg(seed_iso_path.to_string_lossy().to_string().as_str())
                .arg("-V")
                .arg("cidata")
                .arg("user-data")
                .arg("network-config")
                .arg("meta-data")
                .current_dir(t.path())
                .output()?;

            if !mkisofs_out.status.success() {
                return Err(Error::MkisofsError(String::from_utf8(mkisofs_out.stderr)?));
            }

            // Zoneadm boot -z [vm_name]
            zone_adm_handle.boot_blocking()?;

            Ok(VPCResponse::Empty)
        }
        VPCAction::Uninstall { tenant, name } => todo!(),
    }
}

fn handle_remote(_action: &VPCAction) -> Result<VPCResponse> {
    todo!()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct UserData {
    vyos_config_commands: Vec<String>,
}

impl ToString for UserData {
    fn to_string(&self) -> String {
        let content = serde_yaml::to_string(&self).unwrap();
        format!("#cloud-config\n{}", content)
    }
}

impl UserData {
    pub fn new() -> Self {
        Self {
            vyos_config_commands: vec![],
        }
    }

    pub fn set(&mut self, args: &[&str]) -> &mut Self {
        self.vyos_config_commands
            .push(format!("set {}", args.join(" ")));
        self
    }

    pub fn delete(&mut self, args: &[&str]) -> &mut Self {
        self.vyos_config_commands
            .push(format!("delete {}", args.join(" ")));
        self
    }
}

pub(crate) fn setup_vnic_for_router(
    backing_nic_name: &str,
    net_name: &str,
    external: bool,
) -> Result<String> {
    let vnic_name = if external {
        format!("{}_rt0", net_name)
    } else {
        format!("{}_rt1", net_name)
    };

    let dladm_out = Command::new("dladm")
        .arg("create-vnic")
        .arg("-l")
        .arg(backing_nic_name)
        .arg(&vnic_name)
        .output()?;

    if !dladm_out.status.success() {
        println!("{}", String::from_utf8_lossy(&dladm_out.stderr));
    }

    Ok(vnic_name)
}

pub(crate) fn get_vswitch_name(id: u64, name: &str) -> String {
    format!("{}{}", name, id)
}

pub(crate) fn create_router_vm(net: (u64, vpc::VPC), root_disk_vol_name: String) {}

pub(crate) fn get_vpc(
    db: &bonsaidb::local::Database,
    tenant: &uuid::Uuid,
    name: &str,
) -> Result<(u64, vpc::VPC)> {
    let resuting_list: Vec<(u64, vpc::VPC)> = vpc::VPC::all(db)
        .query()?
        .into_iter()
        .filter_map(|doc| {
            if (&doc.contents.tenant_id == tenant) && (&doc.contents.name == name) {
                Some((doc.header.id, doc.contents))
            } else {
                None
            }
        })
        .collect();
    if resuting_list.len() != 1 {
        Err(Error::DuplicateVPCS)
    } else {
        Ok(resuting_list[0].clone())
    }
}
