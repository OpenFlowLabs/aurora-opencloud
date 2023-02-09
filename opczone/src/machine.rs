use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};

use crate::brand::Brand;
use crate::vmext::write_brand_config;
use anyhow::Result;
use common::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ZONE_IDENT_LEN: usize = 6;

fn default_to_false() -> bool {
    false
}

fn default_dns_domain() -> String {
    "local".into()
}

fn default_lwps() -> u32 {
    2000
}

fn default_mdata_exec_timeout() -> u32 {
    300
}

fn default_ram() -> u32 {
    256
}

fn default_vcpus() -> u32 {
    1
}

fn default_virtio_txburst() -> u32 {
    128
}

fn default_virtio_txtimer() -> u32 {
    200000
}

fn default_zfs_io_priority() -> u32 {
    100
}

fn default_quota() -> u32 {
    10
}

fn default_cpu_shares() -> u32 {
    100
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BlockSize(u32);

impl Default for BlockSize {
    fn default() -> Self {
        Self(8192)
    }
}

impl BlockSize {
    pub fn default_zfs_recsize() -> Self {
        Self(131072)
    }
}

fn deserialize_block_size<'de, D>(deserializer: D) -> Result<BlockSize, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let blk_size = u32::deserialize(deserializer)?;

    if blk_size < 512 {
        return Err(serde::de::Error::custom(
            "block size too small must be at least 512",
        ));
    }

    if blk_size > 131072 {
        return Err(serde::de::Error::custom(
            "block size too big must be under 131072",
        ));
    }

    if (blk_size & (blk_size - 1)) != 0 {
        return Err(serde::de::Error::custom("block size not power of 2"));
    }

    Ok(BlockSize(blk_size))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum DiskCompressionMethods {
    On,
    Off,
    Gzip,
    Lz4,
    Lzjb,
    Zle,
}

impl Default for DiskCompressionMethods {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum DiskMedia {
    Disk,
    CDrom,
}

impl Default for DiskMedia {
    fn default() -> Self {
        Self::Disk
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum DiskModel {
    Virtio,
    Ide,
    Scsi,
}

impl Default for DiskModel {
    fn default() -> Self {
        Self::Virtio
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DiskPayload {
    pub name: String,
    pub disk_driver: Option<DiskModel>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddDiskPayload {
    #[serde(default, deserialize_with = "deserialize_block_size")]
    pub block_size: BlockSize,
    #[serde(default = "default_to_false")]
    pub boot: bool,
    #[serde(default)]
    pub compression: DiskCompressionMethods,
    #[serde(default = "default_to_false")]
    pub nocreate: bool,
    pub image_name: Option<String>,
    pub image_size: Option<i32>,
    pub image_uuid: Option<uuid::Uuid>,
    pub refreservation: Option<i32>,
    pub size: Option<i32>,
    #[serde(default)]
    pub media: DiskMedia,
    pub model: Option<DiskModel>,
    pub zpool: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateDiskPayload {
    pub boot: Option<bool>,
    pub compression: Option<DiskCompressionMethods>,
    pub image_name: Option<String>,
    pub image_size: Option<i32>,
    pub image_uuid: Option<uuid::Uuid>,
    pub refreservation: Option<i32>,
    pub size: Option<i32>,
    pub media: Option<DiskMedia>,
    pub model: Option<DiskModel>,
    pub zpool: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddFileSystemPayload {
    #[serde(rename = "type")]
    pub fs_type: String,
    pub source: String,
    pub target: String,
    pub raw: String,
    pub options: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum NicModel {
    Virtio,
    E1000,
    Rtl8139,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddNicPayload {
    #[serde(default = "default_to_false")]
    pub allow_dhcp_spoofing: bool,
    #[serde(default = "default_to_false")]
    pub allow_ip_spoofing: bool,
    #[serde(default = "default_to_false")]
    pub allow_mac_spoofing: bool,
    #[serde(default = "default_to_false")]
    pub allow_restricted_traffic: bool,
    #[serde(default = "default_to_false")]
    pub allow_unfiltered_promisc: bool,
    pub blocked_outgoing_ports: Option<Vec<i32>>,
    pub allowed_ips: Option<Vec<String>>,
    #[serde(default = "default_to_false")]
    pub dhcp_server: bool,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub ip: Option<String>,
    pub mac: Option<String>,
    pub model: Option<NicModel>,
    pub netmask: Option<String>,
    pub network_uuid: Option<uuid::Uuid>,
    pub nic_tag: Option<String>,
    #[serde(default = "default_to_false")]
    pub primary: bool,
    pub vlan_id: Option<i32>,
    pub vrrp_primary_ip: Option<String>,
    pub vrrp_vrid: Option<u8>,
}

impl Default for AddNicPayload {
    fn default() -> Self {
        Self {
            allow_dhcp_spoofing: false,
            allow_ip_spoofing: false,
            allow_mac_spoofing: false,
            allow_restricted_traffic: false,
            allow_unfiltered_promisc: false,
            blocked_outgoing_ports: None,
            allowed_ips: None,
            dhcp_server: false,
            gateway: None,
            interface: None,
            ip: None,
            mac: None,
            model: None,
            netmask: None,
            network_uuid: None,
            nic_tag: None,
            primary: false,
            vlan_id: None,
            vrrp_primary_ip: None,
            vrrp_vrid: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateNicPayload {
    pub allow_dhcp_spoofing: Option<bool>,
    pub allow_ip_spoofing: Option<bool>,
    pub allow_mac_spoofing: Option<bool>,
    pub allow_restricted_traffic: Option<bool>,
    pub allow_unfiltered_promisc: Option<bool>,
    pub blocked_outgoing_ports: Option<Vec<i32>>,
    pub allowed_ips: Option<Vec<String>>,
    pub dhcp_server: Option<bool>,
    pub gateway: Option<String>,
    pub ip: Option<String>,
    pub model: Option<NicModel>,
    pub netmask: Option<String>,
    pub network_uuid: Option<uuid::Uuid>,
    pub nic_tag: String,
    pub primary: Option<bool>,
    pub vlan_id: Option<i32>,
    pub vrrp_primary_ip: Option<String>,
    pub vrrp_vrid: Option<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct OnDiskNicPayload {
    pub allow_dhcp_spoofing: bool,
    pub allow_ip_spoofing: bool,
    pub allow_mac_spoofing: bool,
    pub allow_restricted_traffic: bool,
    pub allow_unfiltered_promisc: bool,
    pub blocked_outgoing_ports: Option<Vec<i32>>,
    pub allowed_ips: Vec<String>,
    pub dhcp_server: bool,
    pub gateway: Option<String>,
    // name of the VNIC
    pub interface: String,
    pub mac: Option<String>,
    pub model: Option<NicModel>,
    pub network_uuid: Option<uuid::Uuid>,
    pub nic_tag: String,
    pub primary: bool,
    pub vlan_id: Option<i32>,
    pub vrrp_primary_ip: Option<String>,
    pub vrrp_vrid: Option<u8>,
}

impl From<AddNicPayload> for OnDiskNicPayload {
    fn from(payload: AddNicPayload) -> Self {
        Self {
            allow_dhcp_spoofing: payload.allow_dhcp_spoofing,
            allow_ip_spoofing: payload.allow_ip_spoofing,
            allow_mac_spoofing: payload.allow_mac_spoofing,
            allow_restricted_traffic: payload.allow_restricted_traffic,
            allow_unfiltered_promisc: payload.allow_unfiltered_promisc,
            blocked_outgoing_ports: payload.blocked_outgoing_ports,
            allowed_ips: if let Some(allowed_ips) = payload.allowed_ips {
                allowed_ips
            } else {
                let ip = payload.ip.unwrap_or_default();
                vec![ip]
            },
            dhcp_server: payload.dhcp_server,
            gateway: payload.gateway,
            interface: if let Some(iface) = payload.interface {
                iface
            } else {
                new_random_interface_name()
            },
            mac: payload.mac,
            model: payload.model,
            network_uuid: payload.network_uuid,
            nic_tag: if let Some(nic_tag) = payload.nic_tag {
                nic_tag
            } else {
                String::new()
            },
            primary: payload.primary,
            vlan_id: payload.vlan_id,
            vrrp_primary_ip: payload.vrrp_primary_ip,
            vrrp_vrid: payload.vrrp_vrid,
        }
    }
}

fn new_random_interface_name() -> String {
    let mut rng = rand::thread_rng();
    let zone_ident_str: String = (0..ZONE_IDENT_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("z{}n0", zone_ident_str)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum VMGraphicsKind {
    Cirrus,
    Std,
    Vmware,
    Qxl,
    Xenfb,
}

impl Default for VMGraphicsKind {
    fn default() -> Self {
        Self::Std
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Payload {
    Create(CreatePayload),
    Update(UpdatePayload),
    OnDisk(OnDiskPayload),
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct OnDiskPayload {
    pub alias: Option<String>,
    pub archive_on_delete: bool,
    pub billing_id: Option<uuid::Uuid>,
    pub boot: Option<String>,
    pub cpu_type: Option<String>,
    pub customer_metadata: Option<Value>,
    pub disks: Vec<DiskPayload>,
    pub disk_driver: DiskModel,
    pub do_not_inventory: bool,
    pub dns_domain: String,
    pub firewall_enabled: bool,
    pub hostname: Option<String>,
    pub internal_metadata: Option<Value>,
    pub maintain_resolvers: bool,
    pub mdata_exec_timeout: u32,
    pub nics: Vec<OnDiskNicPayload>,
    pub nic_driver: Option<NicModel>,
    pub nowait: bool,
    pub owner_uuid: Option<uuid::Uuid>,
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub qemu_opts: Option<String>,
    pub qemu_extra_opts: Option<String>,
    pub resolvers: Option<Vec<String>>,
    pub routes: Option<HashMap<String, String>>,
    pub tmpfs: Option<u32>,
    pub uuid: uuid::Uuid,
    pub vcpus: u32,
    pub vga: VMGraphicsKind,
    pub virtio_txburst: u32,
    pub virtio_txtimer: u32,
    pub vnc_password: Option<String>,
    pub vnc_port: Option<u32>,
}

impl From<CreatePayload> for OnDiskPayload {
    fn from(payload: CreatePayload) -> Self {
        Self {
            alias: payload.alias,
            archive_on_delete: payload.archive_on_delete,
            billing_id: payload.billing_id,
            boot: payload.boot,
            cpu_type: payload.cpu_type,
            customer_metadata: payload.customer_metadata,
            disks: vec![],
            disk_driver: payload.disk_driver,
            do_not_inventory: payload.do_not_inventory,
            dns_domain: payload.dns_domain,
            firewall_enabled: payload.firewall_enabled,
            hostname: payload.hostname,
            internal_metadata: payload.internal_metadata,
            maintain_resolvers: payload.maintain_resolvers,
            mdata_exec_timeout: payload.mdata_exec_timeout,
            nics: if let Some(nics) = payload.nics {
                nics.into_iter().map(|n| n.into()).collect()
            } else {
                vec![]
            },
            nic_driver: payload.nic_driver,
            nowait: payload.nowait,
            owner_uuid: payload.owner_uuid,
            package_name: payload.package_name,
            package_version: payload.package_version,
            qemu_opts: payload.qemu_opts,
            qemu_extra_opts: payload.qemu_extra_opts,
            resolvers: payload.resolvers,
            routes: payload.routes,
            tmpfs: payload.tmpfs,
            uuid: if let Some(uuid) = payload.uuid {
                uuid
            } else {
                uuid::Uuid::nil()
            },
            vcpus: payload.vcpus,
            vga: payload.vga,
            virtio_txburst: payload.virtio_txburst,
            virtio_txtimer: payload.virtio_txtimer,
            vnc_password: payload.vnc_password,
            vnc_port: payload.vnc_port,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreatePayload {
    pub alias: Option<String>,
    #[serde(default = "default_to_false")]
    pub archive_on_delete: bool,
    #[serde(default = "default_to_false")]
    pub autoboot: bool,
    pub billing_id: Option<uuid::Uuid>,
    pub boot: Option<String>,
    pub brand: Brand,
    pub cpu_cap: Option<u32>,
    #[serde(default = "default_cpu_shares")]
    pub cpu_shares: u32,
    pub cpu_type: Option<String>,
    pub customer_metadata: Option<Value>,
    pub image_uuid: Option<uuid::Uuid>,
    #[serde(default = "default_to_false")]
    pub delegate_dataset: bool,
    pub disks: Option<Vec<AddDiskPayload>>,
    #[serde(default)]
    pub disk_driver: DiskModel,
    #[serde(default = "default_to_false")]
    pub do_not_inventory: bool,
    #[serde(default = "default_dns_domain")]
    pub dns_domain: String,
    pub filesystems: Option<Vec<AddFileSystemPayload>>,
    #[serde(default = "default_to_false")]
    pub firewall_enabled: bool,
    pub fs_allowed: Option<String>,
    pub hostname: Option<String>,
    pub internal_metadata: Option<Value>,
    #[serde(default = "default_to_false")]
    pub indestructible_delegated: bool,
    #[serde(default = "default_to_false")]
    pub indestructible_zoneroot: bool,
    pub limit_priv: Option<String>,
    #[serde(default = "default_to_false")]
    pub maintain_resolvers: bool,
    pub max_locked_memory: Option<u32>,
    #[serde(default = "default_lwps")]
    pub max_lwps: u32,
    pub max_physical_memory: Option<u32>,
    pub max_swap: Option<u32>,
    #[serde(default = "default_mdata_exec_timeout")]
    pub mdata_exec_timeout: u32,
    pub nics: Option<Vec<AddNicPayload>>,
    pub nic_driver: Option<NicModel>,
    #[serde(default = "default_to_false")]
    pub nowait: bool,
    pub owner_uuid: Option<uuid::Uuid>,
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub qemu_opts: Option<String>,
    pub qemu_extra_opts: Option<String>,
    #[serde(default = "default_quota")]
    pub quota: u32,
    #[serde(default = "default_ram")]
    pub ram: u32,
    pub resolvers: Option<Vec<String>>,
    pub routes: Option<HashMap<String, String>>,
    pub spice_opts: Option<String>,
    pub spice_password: Option<String>,
    pub spice_port: Option<u32>,
    pub tmpfs: Option<u32>,
    pub uuid: Option<uuid::Uuid>,
    #[serde(default = "default_vcpus")]
    pub vcpus: u32,
    #[serde(default)]
    pub vga: VMGraphicsKind,
    #[serde(default = "default_virtio_txburst")]
    pub virtio_txburst: u32,
    #[serde(default = "default_virtio_txtimer")]
    pub virtio_txtimer: u32,
    pub vnc_password: Option<String>,
    pub vnc_port: Option<u32>,
    pub zfs_data_compression: Option<DiskCompressionMethods>,
    #[serde(default = "BlockSize::default_zfs_recsize")]
    pub zfs_data_recsize: BlockSize,
    #[serde(default = "default_zfs_io_priority")]
    pub zfs_io_priority: u32,
    pub zfs_root_compression: Option<DiskCompressionMethods>,
    #[serde(default = "BlockSize::default_zfs_recsize")]
    pub zfs_root_recsize: BlockSize,
    pub zonename: Option<String>,
    pub zpool: Option<String>,
}

impl Default for CreatePayload {
    fn default() -> Self {
        CreatePayload {
            image_uuid: None,
            quota: default_quota(),
            alias: None,
            archive_on_delete: false,
            autoboot: false,
            billing_id: None,
            boot: None,
            brand: Brand::Image,
            cpu_cap: None,
            cpu_shares: default_cpu_shares(),
            cpu_type: None,
            customer_metadata: None,
            delegate_dataset: false,
            disks: None,
            disk_driver: DiskModel::default(),
            do_not_inventory: false,
            dns_domain: default_dns_domain(),
            filesystems: None,
            firewall_enabled: false,
            fs_allowed: None,
            hostname: None,
            internal_metadata: None,
            indestructible_delegated: false,
            indestructible_zoneroot: false,
            limit_priv: None,
            maintain_resolvers: false,
            max_locked_memory: None,
            max_lwps: default_lwps(),
            max_physical_memory: None,
            max_swap: None,
            mdata_exec_timeout: default_mdata_exec_timeout(),
            nics: None,
            nic_driver: None,
            nowait: false,
            owner_uuid: None,
            package_name: None,
            package_version: None,
            qemu_opts: None,
            qemu_extra_opts: None,
            ram: default_ram(),
            resolvers: None,
            routes: None,
            spice_opts: None,
            spice_password: None,
            spice_port: None,
            tmpfs: None,
            uuid: None,
            vcpus: default_vcpus(),
            vga: VMGraphicsKind::default(),
            virtio_txburst: default_virtio_txburst(),
            virtio_txtimer: default_virtio_txtimer(),
            vnc_password: None,
            vnc_port: None,
            zfs_data_compression: None,
            zfs_data_recsize: BlockSize::default(),
            zfs_io_priority: default_zfs_io_priority(),
            zfs_root_compression: None,
            zfs_root_recsize: BlockSize::default_zfs_recsize(),
            zonename: None,
            zpool: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdatePayload {
    pub alias: Option<String>,
    pub archive_on_delete: Option<bool>,
    pub autoboot: Option<bool>,
    pub billing_id: Option<uuid::Uuid>,
    pub boot: Option<String>,
    pub cpu_cap: Option<i32>,
    pub cpu_shares: Option<i32>,
    pub cpu_type: Option<String>,
    pub customer_metadata: Option<Value>,
    pub disk_driver: Option<DiskModel>,
    pub do_not_inventory: Option<bool>,
    pub firewall_enabled: Option<bool>,
    pub fs_allowed: Option<String>,
    pub hostname: Option<String>,
    pub internal_metadata: Option<Value>,
    pub indestructible_delegated: Option<bool>,
    pub indestructible_zoneroot: Option<bool>,
    pub kernel_version: Option<String>,
    pub limit_priv: Option<String>,
    pub maintain_resolvers: Option<bool>,
    pub max_locked_memory: Option<i32>,
    pub max_lwps: Option<i32>,
    pub max_physical_memory: Option<i32>,
    pub max_swap: Option<i32>,
    pub nic_driver: Option<NicModel>,
    pub add_nics: Option<Vec<AddNicPayload>>,
    pub update_nics: Option<Vec<UpdateNicPayload>>,
    pub remove_nics: Option<Vec<String>>,
    pub owner_uuid: Option<uuid::Uuid>,
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub qemu_opts: Option<String>,
    pub qemu_extra_opts: Option<String>,
    pub quota: Option<i32>,
    pub ram: Option<i32>,
    pub resolvers: Option<Vec<String>>,
    pub routes: Option<HashMap<String, String>>,
    pub spice_opts: Option<String>,
    pub spice_password: Option<String>,
    pub spice_port: Option<i32>,
    pub tmpfs: Option<i32>,
    pub virtio_txburst: Option<i32>,
    pub virtio_txtimer: Option<i32>,
    pub vnc_password: Option<String>,
    pub vnc_port: Option<i32>,
    pub zfs_data_compression: Option<DiskCompressionMethods>,
    pub zfs_data_recsize: Option<BlockSize>,
    pub zfs_io_priority: Option<i32>,
    pub zfs_root_compression: Option<DiskCompressionMethods>,
    pub zfs_root_recsize: Option<BlockSize>,
    pub add_disks: Option<Vec<AddDiskPayload>>,
    pub update_disks: Option<Vec<UpdateDiskPayload>>,
    pub remove_disks: Option<Vec<String>>,
}

/// This function mainly executes zoneadm and puts the json into a location the install and statechange
/// hook can take over
pub fn define_vm(payload: CreatePayload) -> Result<OnDiskPayload> {
    let zone_uuid = if let Some(uuid) = payload.uuid {
        uuid
    } else {
        uuid::Uuid::new_v4()
    };

    let mut cfg = zone::Config::create(zone_uuid.to_string(), true, zone::CreationOptions::Blank);

    //We assume /zones is the dataset where all zones should be located
    cfg.get_global()
        .set_brand(payload.brand.to_string())
        .set_ip_type(zone::IpType::Exclusive)
        .set_path(Path::new("/zones").join(zone_uuid.to_string()))
        .set_max_lwps(Some(payload.max_lwps))
        .set_cpu_shares(Some(payload.cpu_shares));

    if let Some(cpu_cap) = payload.cpu_cap {
        let caps = zone::CappedCpu {
            ncpus: cpu_cap as f64 / 100.0,
        };
        cfg.add_capped_cpu(&caps);
    }

    if payload.delegate_dataset {
        let ds = zone::Dataset {
            name: format!("/zones/{}/data", zone_uuid.to_string()),
        };
        cfg.add_dataset(&ds);
    }

    if let Some(filesystems) = &payload.filesystems {
        for fs in filesystems {
            // dir, special, raw, type, options
            let raw = if !fs.raw.is_empty() {
                Some(fs.raw.clone())
            } else {
                None
            };

            let f = zone::Fs {
                ty: fs.fs_type.clone(),
                dir: fs.target.clone(),
                special: fs.source.clone(),
                raw: raw,
                options: fs.options.clone(),
            };
            cfg.add_fs(&f);
        }
    }

    if let Some(limit_priv) = payload.limit_priv.clone() {
        let mut privset = BTreeSet::new();
        for p in limit_priv.split(",") {
            privset.insert(p.clone().to_owned());
        }

        cfg.get_global().set_limitpriv(privset);
    }

    let capped_memory = if let Some(mut max_physical_memory) = &payload.max_physical_memory {
        if max_physical_memory < payload.ram {
            if payload.brand == Brand::Propolis || payload.brand == Brand::Bhyve {
                max_physical_memory = payload.ram + 1024;
            } else {
                max_physical_memory = payload.ram;
            }
        }
        max_physical_memory
    } else {
        payload.ram
    };

    let mut mem_cap = zone::CappedMemory {
        physical: Some(format!("{}M", capped_memory.to_string())),
        ..Default::default()
    };

    if let Some(max_locked_memory) = payload.max_locked_memory {
        mem_cap.locked = Some(max_locked_memory.to_string());
    }

    if let Some(max_swap) = payload.max_swap {
        mem_cap.swap = Some(max_swap.to_string());
    }
    cfg.add_capped_memory(&mem_cap);

    let mut disk_payload: OnDiskPayload = payload.into();
    disk_payload.uuid = zone_uuid;

    for nic in &disk_payload.nics {
        let mut nic_opts = zone::Net {
            physical: nic.interface.clone(),
            ..Default::default()
        };

        if nic.allowed_ips.len() > 0 {
            nic_opts.allowed_address = Some(nic.allowed_ips[0].clone());
        }
        if let Some(gateway) = nic.gateway.clone() {
            nic_opts.default_router = Some(gateway);
        }

        cfg.add_net(&nic_opts);
    }

    cfg.run()?;

    info!(target: "define_vm", "defining VM: {}", zone_uuid.to_string());

    write_brand_config(&disk_payload)?;

    Ok(disk_payload)
}

pub fn update_vm(_payload: UpdatePayload) -> Result<()> {
    todo!()
}
