use bonsaidb::core::schema::Collection;
use ipnet::{Ipv4Net, Ipv6Net};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    UUidError(#[from] uuid::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    StdAddrParseError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    AddrParseError(#[from] ipnet::AddrParseError),

    #[error("the gateway address is not in the correct Ip Stack")]
    GatewayNotCorrectStack,

    #[error("ip pool exhausted")]
    IPPoolExhausted,
}

type Result<T> = miette::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum Net {
    V6Only(Ipv6Net),
    DualStack { v6: Ipv6Net, v4: Ipv4Net },
}

impl Net {
    pub fn net_v6only_from_string(v6: &str) -> Result<Net> {
        Ok(Self::V6Only(v6.parse()?))
    }

    pub fn net_dualstack_from_string(v4: &str, v6: &str) -> Result<Net> {
        Ok(Self::DualStack {
            v6: v6.parse()?,
            v4: v4.parse()?,
        })
    }
}

impl Default for Net {
    fn default() -> Self {
        Self::V6Only("fc00::/126".parse().unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Addr {
    V6Only(Ipv6Addr),
    DualStack { v6: Ipv6Addr, v4: Ipv4Addr },
}

impl Addr {
    pub fn v6only_from_string(a: &str) -> Result<Addr> {
        Ok(Addr::V6Only(a.parse()?))
    }

    pub fn dual_stack_from_string(v4: &str, v6: &str) -> Result<Addr> {
        Ok(Addr::DualStack {
            v6: v6.parse()?,
            v4: v4.parse()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ReservationType {
    DHCP,
    Static,
    Dual {
        dhcp_start: IpAddr,
        dhcp_length: u128,
    },
}

#[derive(Debug, Deserialize, Serialize, Collection, PartialEq, Eq, Clone)]
#[collection(name = "vpc")]
pub struct VPC {
    pub tenant_id: Uuid,
    pub name: String,
    tenant_nic_suffix: Option<String>,
    net: Net,
    default_gateway: Option<Addr>,
    default_gateway_uuid: Option<Uuid>,
    reservation_type: ReservationType,
    reservations: Vec<IpAddr>,
}

impl VPC {
    pub(crate) fn new(tenant_id: Uuid, name: &str) -> Self {
        let net = Net::default();

        Self {
            tenant_id,
            name: name.to_owned(),
            tenant_nic_suffix: None,
            net,
            default_gateway: None,
            default_gateway_uuid: None,
            reservation_type: ReservationType::Static,
            reservations: vec![],
        }
    }

    pub fn add_gateway(&mut self, addr: Option<Addr>) -> Result<()> {
        let default_gateway = if let Some(addr_string) = addr {
            addr_string
        } else {
            match &self.net {
                Net::V6Only(v6) => Addr::V6Only(v6.hosts().nth(1).unwrap()),
                Net::DualStack { v6, v4 } => Addr::DualStack {
                    v6: v6.hosts().nth(1).unwrap(),
                    v4: v4.hosts().nth(1).unwrap(),
                },
            }
        };

        match &self.net {
            Net::V6Only(_) => {
                if matches!(default_gateway, Addr::V6Only { .. }) {
                    Ok(())
                } else {
                    Err(Error::GatewayNotCorrectStack)
                }
            }
            Net::DualStack { .. } => {
                if matches!(default_gateway, Addr::DualStack { .. }) {
                    Ok(())
                } else {
                    Err(Error::GatewayNotCorrectStack)
                }
            }
        }?;

        self.default_gateway = Some(default_gateway.clone());

        match default_gateway {
            Addr::V6Only(addr) => {
                self.reservations.push(IpAddr::V6(addr));
            }
            Addr::DualStack { v6, v4 } => {
                self.reservations.push(IpAddr::V4(v4));
                self.reservations.push(IpAddr::V6(v6));
            }
        }

        Ok(())
    }

    pub fn reserve_new_address(&mut self) -> Result<Addr> {
        let addr = match self.net {
            Net::V6Only(v6net) => Addr::V6Only(
                v6net
                    .hosts()
                    .filter(|i| !self.reservations.contains(&IpAddr::V6(i.clone())))
                    .collect::<Vec<Ipv6Addr>>()
                    .first()
                    .ok_or(Error::IPPoolExhausted)?
                    .clone(),
            ),
            Net::DualStack { v6, v4 } => Addr::DualStack {
                v6: v6
                    .hosts()
                    .filter(|i| !self.reservations.contains(&IpAddr::V6(i.clone())))
                    .collect::<Vec<Ipv6Addr>>()
                    .first()
                    .ok_or(Error::IPPoolExhausted)?
                    .clone(),
                v4: v4
                    .hosts()
                    .filter(|i| !self.reservations.contains(&IpAddr::V4(i.clone())))
                    .collect::<Vec<Ipv4Addr>>()
                    .first()
                    .ok_or(Error::IPPoolExhausted)?
                    .clone(),
            },
        };

        match &addr {
            Addr::V6Only(v6addr) => self.reservations.push(IpAddr::V6(v6addr.clone())),
            Addr::DualStack { v6, v4 } => {
                self.reservations.push(IpAddr::V6(v6.clone()));
                self.reservations.push(IpAddr::V4(v4.clone()))
            }
        };

        Ok(addr)
    }
}

pub struct Builder {
    vpc: VPC,
}

impl Builder {
    pub fn new(tenant_id: &Uuid, name: &str) -> Self {
        Self {
            vpc: VPC::new(tenant_id.clone(), name),
        }
    }

    pub fn set_net(&mut self, net: Net) -> &mut Self {
        self.vpc.net = net;
        self
    }

    pub fn into_vpc(self) -> VPC {
        self.vpc
    }
}

pub type VarpdRoutingFile = BTreeMap<String, VarpdRoutingEntry>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum VarpdRoutingEntry {
    V6Only {
        ip: String,
        port: i32,
        ndp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dhcp_proxy: Option<String>,
    },
    DualStack {
        ip: String,
        port: i32,
        arp: String,
        ndp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dhcp_proxy: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4();
        let vpc = Builder::new(&id, "testnet").into_vpc();
        assert_eq!(vpc.name, "testnet".to_string());
    }

    #[test]
    fn serialize_routing_file() -> miette::Result<()> {
        let entries: VarpdRoutingFile = VarpdRoutingFile::from([
            (
                "de:ad:be:ef:00:00".to_string(),
                VarpdRoutingEntry::DualStack {
                    arp: "10.55.55.2".to_string(),
                    ip: "10.88.88.69".to_string(),
                    ndp: "fe80::3".to_string(),
                    port: 4789,
                    dhcp_proxy: None,
                },
            ),
            (
                "de:ad:be:ef:00:01".to_string(),
                VarpdRoutingEntry::DualStack {
                    arp: "10.55.55.3".to_string(),
                    dhcp_proxy: Some("de:ad:be:ef:00:00".to_string()),
                    ip: "10.88.88.70".to_string(),
                    ndp: "fe80::4".to_string(),
                    port: 4789,
                },
            ),
            (
                "de:ad:be:ef:00:02".to_string(),
                VarpdRoutingEntry::DualStack {
                    arp: "10.55.55.4".to_string(),
                    ip: "10.88.88.71".to_string(),
                    ndp: "fe80::5".to_string(),
                    port: 4789,
                    dhcp_proxy: None,
                },
            ),
        ]);

        let json = serde_json::to_string_pretty(&entries).into_diagnostic()?;

        let expected = read_to_string("testdata/test.json").into_diagnostic()?;

        assert_eq!(expected, json);

        Ok(())
    }
}
