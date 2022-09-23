use anyhow::{Result, bail};
use std::{
    collections::HashMap,
    path::{PathBuf},
};

pub mod bundle;

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Document {
    #[knuffel(child, unwrap(argument))]
    pub author: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub name: String,
    #[knuffel(child, unwrap(argument), default = 1)]
    pub version: i32,
    #[knuffel(child, unwrap(argument))]
    pub base_on: Option<String>,
    #[knuffel(children)]
    pub actions: Vec<Action>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub enum Action {
    Volume(Volume),
    Remove(#[knuffel(argument)] String),
    ExtractTarball(#[knuffel(argument)] String),
    InitializeDevfs,
    AssembleFile(AssembleFile),
    Group(#[knuffel(argument)] String),
    User(
        #[knuffel(argument)] String,
        #[knuffel(argument)] Option<String>,
    ),
    Symlink(Symlink),
    Dir(Dir),
    File(File),
    Perm(Dir),
    Ips(Ips),
    SeedSmf,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Ips {
    #[knuffel(children)]
    pub actions: Vec<IpsActions>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct IpsProperties {
    #[knuffel(properties)]
    pub properties: HashMap<String, String>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct IpsPackageList {
    #[knuffel(arguments)]
    pub packages: Vec<String>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub enum IpsActions {
    InitializeImage,
    InstallPackages(IpsPackageList),
    InstallOptionals,
    SetProperty(IpsProperties),
    SetPublisher(IpsPublisher),
    ApprovePublisherCA(CaCertificates),
    UninstallPackages(IpsPackageList),
    SetVariant(IpsProperties),
    SetFacet(IpsProperties),
    PurgeHistory,
    SetMediator(Mediator),
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Mediator {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(property)]
    pub implementation: Option<String>,
    #[knuffel(property)]
    pub version: Option<String>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct CaCertificates {
    #[knuffel(argument)]
    pub publisher: String,
    #[knuffel(argument)]
    pub cert_file: String,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct IpsPublisher {
    #[knuffel(argument)]
    pub publisher: String,
    #[knuffel(arguments)]
    pub uris: Vec<String>,
}

#[derive(knuffel::Decode, Clone, Default, Debug, PartialEq)]
pub struct CommonPerms {
    #[knuffel(child, unwrap(argument))]
    pub owner: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub group: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub mode: Option<i32>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Dir {
    #[knuffel(flatten(child))]
    pub common: CommonPerms,
    #[knuffel(argument)]
    pub path: PathBuf,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct File {
    #[knuffel(flatten(child))]
    pub common: CommonPerms,
    #[knuffel(child, unwrap(argument))]
    pub src: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub content: Option<String>,
    #[knuffel(child)]
    pub is_template: bool,
    #[knuffel(argument)]
    pub path: PathBuf,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Symlink {
    #[knuffel(argument)]
    pub link: PathBuf,
    #[knuffel(argument)]
    pub target: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub owner: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub group: Option<String>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Volume {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub mountpoint: Option<String>,
    #[knuffel(children)]
    pub properties: Vec<VolumeProperty>,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct VolumeProperty {
    #[knuffel(node_name)]
    pub name: String,
    #[knuffel(argument)]
    pub value: String,
    #[knuffel(type_name, default = "")]
    pub driver_name: String,
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct AssembleFile {
    #[knuffel(flatten(child))]
    pub common: CommonPerms,
    #[knuffel(child, unwrap(argument))]
    pub dir: PathBuf,
    #[knuffel(argument)]
    pub output: PathBuf,
    #[knuffel(child, unwrap(argument))]
    pub prefix: Option<String>,
}

pub fn run_action(root: &str, action: Action) -> Result<()> {
    match action {
        Action::Volume(_) => todo!(),
        Action::Remove(_) => todo!(),
        Action::ExtractTarball(_) => todo!(),
        Action::InitializeDevfs => todo!(),
        Action::AssembleFile(_) => todo!(),
        Action::Group(_) => todo!(),
        Action::User(_, _) => todo!(),
        Action::Symlink(_) => todo!(),
        Action::Dir(_) => todo!(),
        Action::File(_) => todo!(),
        Action::Perm(_) => todo!(),
        Action::Ips(ips_actions) => {
            for action in ips_actions.actions {
                run_ips_action(root, action)?;
            }
            Ok(())
        },
        Action::SeedSmf => todo!(),
    }
}

pub fn run_ips_action(root: &str, action: IpsActions) -> Result<()> {
    match action {
        IpsActions::InitializeImage => {
            illumos_image_builder::pkg(&[
                "image-create",
                "--full",
                root,
            ])
        },
        IpsActions::InstallPackages(pkgs) => {
            illumos_image_builder::pkg_install(root, pkgs.packages.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice())
        },
        IpsActions::InstallOptionals => bail!("install optionals is a not supported operation right now"),
        IpsActions::SetProperty(ips_properties) => {
            for (prop_name, prop_value) in ips_properties.properties {
                illumos_image_builder::pkg(&[
                        "-R",
                        root,
                        "set-property",
                        &prop_name,
                        &prop_value,
                    ])?;
            }
            
            Ok(())
        },
        IpsActions::SetPublisher(pub_props) => {
            let mut args = vec![
                "-R".to_owned(), 
                root.to_string(),
                "set-publisher".to_owned(),
                ];
            for (idx, uri) in pub_props.uris.into_iter().enumerate() {
                if idx == 0 {
                    args.push("-O".to_owned());
                    args.push(uri);
                } else {
                    
                    args.push("-g".to_owned());
                    args.push(uri);
                }
            }

            args.push(pub_props.publisher);

            illumos_image_builder::pkg(args.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice())
        },
        IpsActions::ApprovePublisherCA(_) => bail!("approve ca is a not supported operation right now"),
        IpsActions::UninstallPackages(pkgs) => {
            illumos_image_builder::pkg_uninstall(root, pkgs.packages.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice())
        },
        IpsActions::SetVariant(variant_props) => {
            for (variant_name, variant_value) in variant_props.properties {
                illumos_image_builder::pkg_ensure_variant(
                    root,
                    &variant_name,
                    &variant_value,
                )?;
            }

            Ok(())
        },
        IpsActions::SetFacet(facet_prop) => {
            for (facet_name, facet_value) in facet_prop.properties {
                illumos_image_builder::pkg_ensure_facet(
                    root, 
                    &facet_name, 
                    &facet_value,
                )?;
            }

            Ok(())
        },
        IpsActions::PurgeHistory => {
            illumos_image_builder::pkg(&[
                "-R",
                root,
                "purge-history"
            ])
        },
        IpsActions::SetMediator(mediator_props) => {
            let mut args = vec![
                "-R".to_owned(), 
                root.to_string(),
                "set-mediator".to_owned(),    
            ];
            if let Some(imple) = mediator_props.implementation {
                args.push("-I".to_owned());
                args.push(imple);
            }

            if let Some(vers) = mediator_props.version {
                args.push("-V".to_owned());
                args.push(vers);
            }

            args.push(mediator_props.name);

            illumos_image_builder::pkg(args.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice())
        },
    }
}

mod tests {
    #[test]
    fn it_works() {
        use crate::build::{
            Action, CaCertificates, Document, Ips, IpsPublisher, Mediator, Volume, VolumeProperty,
        };
        use crate::build::{IpsActions, IpsPackageList, IpsProperties};

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
