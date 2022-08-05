pub mod brand;
pub mod definition;
use std::{path::Path, fs::File};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub image: Option<uuid::Uuid>,
    pub quota: i32,
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
            return Ok(mnt.special)
        }
    }

    bail!("No dataset found for mountpoint {}", dname)
}

pub fn get_parent_dataset_path(zonepath: &str) ->Result<&str> {
    let (dname, _) = match common::path_split(zonepath) {
        Some(v) => v,
        None => bail!("could not split zonepath {}", zonepath),
    };
    Ok(dname)
}

pub fn get_config(zonename: &str, zonepath: &str) -> Result<Config> {
    let (parent_ds_path, _) = match common::path_split(zonepath) {
        Some(v) => v,
        None => bail!("could not split zonepath {}", zonepath),
    };

    let config_path = Path::new(parent_ds_path).join("config").join(format!("{}.json", zonename));

    let cfg_file = File::open(config_path)?;

    let cfg = serde_json::from_reader(cfg_file)?;

    Ok(cfg)
}


mod tests {
    #[test]
    fn it_works() {
        use crate::definition::{
            Action, CaCertificates, Document, Ips, IpsPublisher, Mediator, Volume, VolumeProperty,
        };
        use crate::definition::{IpsActions, IpsPackageList, IpsProperties};

        use miette::{Context, IntoDiagnostic};
        use pretty_assertions::assert_eq;
        use std::collections::HashMap;
        use std::fs;

        let file = "testdata/image_base.kdl";

        let comparision = Document {
            author: Some("John Doe <john.doe@example.com>".into()),
            name: "my-image".into(),
            version: 1,
            base_on: Some("img://openindiana.org/hipster".into()),
            actions: vec![
                Action::Volume(Volume {
                    name: "data".into(),
                    mountpoint: Some("/var/lib/pgdata".into()),
                    properties: vec![
                        VolumeProperty {
                            name: "checksum".into(),
                            value: "off".into(),
                            driver_name: "zfs".into(),
                        },
                        VolumeProperty {
                            name: "compression".into(),
                            value: "lz4".into(),
                            driver_name: "zfs".into(),
                        },
                        VolumeProperty {
                            name: "copies".into(),
                            value: "3".into(),
                            driver_name: "zfs".into(),
                        },
                        VolumeProperty {
                            name: "bar".into(),
                            value: "1".into(),
                            driver_name: "foo".into(),
                        },
                    ],
                }),
                Action::Ips(Ips {
                    actions: vec![
                        IpsActions::InstallPackages(IpsPackageList {
                            packages: vec![
                                "developer/gcc-11".into(),
                                "golang".into(),
                                "golang-118".into(),
                            ],
                        }),
                        IpsActions::UninstallPackages(IpsPackageList {
                            packages: vec!["userland-incorportation".into()],
                        }),
                        IpsActions::InstallOptionals,
                        IpsActions::SetProperty(IpsProperties {
                            properties: HashMap::from([("image.prop".into(), "false".into())]),
                        }),
                        IpsActions::SetPublisher(IpsPublisher {
                            publisher: "openindiana.org".into(),
                            uris: vec!["https://pkg.openindiana.org/hipster".into()],
                        }),
                        IpsActions::ApprovePublisherCA(CaCertificates {
                            publisher: "openindiana.org".into(),
                            cert_file: "/path/to/cert/in/image/bundle".into(),
                        }),
                        IpsActions::SetVariant(IpsProperties {
                            properties: HashMap::from([(
                                "opensolaris.zone".into(),
                                "global".into(),
                            )]),
                        }),
                        IpsActions::SetFacet(IpsProperties {
                            properties: HashMap::from([("my.facet.name".into(), "true".into())]),
                        }),
                        IpsActions::SetMediator(Mediator {
                            name: "mysql".into(),
                            implementation: Some("mariadb".into()),
                            version: None,
                        }),
                        IpsActions::PurgeHistory,
                    ],
                }),
            ],
        };

        let text = fs::read_to_string(file)
            .into_diagnostic()
            .wrap_err_with(|| format!("cannot read {:?}", file))
            .unwrap();

        let config = match knuffel::parse::<Document>(file, &text) {
            Ok(config) => config,
            Err(e) => {
                panic!("{:?}", miette::Report::new(e));
            }
        };

        assert_eq!(comparision, config);
    }
}
