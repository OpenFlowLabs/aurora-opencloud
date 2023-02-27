use self::bundle::Bundle;
use crate::{dataset_create_with, get_zone_vroot_dataset};
use anyhow::{bail, Result};
use common::info;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tera::Context;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("{0} not yet supported")]
    NotYetSupported(String),
    #[error("mode must be specified, not found for {0}")]
    NoModeSpecified(String),
    #[error(transparent)]
    AnyHowError(#[from] anyhow::Error),
}

/*
 * Hard-coded user ID and group ID for root:
 */
const ROOT: u32 = 0;

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
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Volume(v) => write!(f, "Action Volume: {}", v.name),
            Action::Remove(r) => write!(f, "Action Remove: {}", r),
            Action::ExtractTarball(t) => write!(f, "Action Extract Tarball: {}", t),
            Action::AssembleFile(fil) => {
                write!(f, "Action Assemble File: {}", fil.output.display())
            }
            Action::Group(g) => write!(f, "Action Ensure Group: {}", g),
            Action::User(u, _) => write!(f, "Action Ensure User: {}", u),
            Action::Symlink(l) => write!(
                f,
                "Action Ensure Symlink: {} -> {}",
                l.target.display(),
                l.link.display()
            ),
            Action::Dir(d) => write!(f, "Action Ensure Directory: {}", d.path.display()),
            Action::File(fil) => write!(f, "Action Ensure File: {}", fil.path.display()),
            Action::Perm(p) => write!(f, "Action Ensure Permissions: {}", p.path.display()),
            Action::Ips(_) => Ok(()),
        }
    }
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

impl std::fmt::Display for IpsProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = vec![];
        for (key, value) in &self.properties {
            v.push(format!("{}={}", key, value))
        }

        if v.len() > 1 {
            let out = v.join(",");
            write!(f, "[{}]", out)
        } else {
            write!(f, "{}", v[0])
        }
    }
}

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct IpsPackageList {
    #[knuffel(arguments)]
    pub packages: Vec<String>,
}

impl std::fmt::Display for IpsPackageList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = self.packages.join(",");
        write!(f, "[{}]", out)
    }
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

impl std::fmt::Display for IpsActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpsActions::InitializeImage => write!(f, "Initialize Image"),
            IpsActions::InstallPackages(p) => write!(f, "Installing packages {}", p),
            IpsActions::InstallOptionals => write!(f, "Install optionals"),
            IpsActions::SetProperty(p) => write!(f, "Setting properties {}", p),
            IpsActions::SetPublisher(publ) => write!(f, "Setting Publisher {}", publ.publisher),
            IpsActions::ApprovePublisherCA(ca) => write!(f, "Approving CA for {}", ca.publisher),
            IpsActions::UninstallPackages(p) => write!(f, "Removing packages {}", p),
            IpsActions::SetVariant(v) => write!(f, "Setting variants {}", v),
            IpsActions::SetFacet(fac) => write!(f, "Set facets {}", fac),
            IpsActions::PurgeHistory => write!(f, "Pruging History"),
            IpsActions::SetMediator(m) => write!(f, "Setting Mediator {}", m.name),
        }
    }
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
    pub mode: Option<u32>,
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

#[derive(knuffel::Decode, Clone, Debug, PartialEq)]
pub struct Smf {
    #[knuffel(child, unwrap(argument), default = false)]
    pub debug: bool,
    #[knuffel(child, unwrap(argument), default = false)]
    pub apply_site: bool,
}

pub fn run_action(zonepath: &str, zonename: &str, bundle: &Bundle, action: Action) -> Result<()> {
    let root_string = if zonename == &zone::current()? {
        zonepath.clone().to_string()
    } else {
        format!("{}/root", zonepath)
    };

    let root = root_string.as_str();
    info!("Running {}", action);
    match action {
        Action::Volume(volume) => {
            if zone::current()? == "global".to_string() {
                panic!("Volume creation is only supported inside a zone")
            }

            let zds = get_zone_vroot_dataset(zonename)?;

            let vds = format!("{}/{}", zds, volume.name);

            let mountpoint = if let Some(p) = volume.mountpoint {
                ("mountpoint".to_string(), p.clone())
            } else {
                ("mountpoint".to_string(), format!("/{}", volume.name))
            };

            let mut props: Vec<(String, String)> = volume
                .properties
                .into_iter()
                .map(|p| (p.name, p.value))
                .collect();

            props.push(mountpoint);

            dataset_create_with(&vds, false, props.as_slice())?;

            Ok(())
        }
        Action::Remove(path) => {
            let rpath = Path::new(root);
            let path = Path::new(&path).strip_prefix("/")?;

            let paths = vec![rpath.join(path)];
            fs_extra::remove_items(&paths)?;

            Ok(())
        }
        Action::ExtractTarball(tarball) => {
            let full_tarball_path = bundle.get_file(&tarball)?;
            crate::run(
                &[
                    "/usr/sbin/tar",
                    "xzeEp@/f",
                    &full_tarball_path.to_string_lossy(),
                    "-C",
                    root,
                ],
                Some(&[]),
            )?;
            Ok(())
        }
        Action::AssembleFile(assemble) => {
            let source_path = bundle.get_file(assemble.dir)?;
            let output_path = Path::new(root).join(assemble.output.strip_prefix("/")?);

            let mut files: Vec<String> = Vec::new();
            let mut diri = std::fs::read_dir(source_path)?;
            while let Some(ent) = diri.next().transpose()? {
                // We keep unwrap here since if this fails something on the system is critically broken
                // I assume so at least.
                if !ent.file_type().unwrap().is_file() {
                    continue;
                }

                let n = ent.file_name();
                let n = n.to_string_lossy().to_string();
                if let Some(ref prefix) = assemble.prefix {
                    if !n.starts_with(prefix.as_str()) {
                        continue;
                    }
                }

                files.push(ent.path().to_str().unwrap().to_string());
            }

            files.sort();

            let mut outstr = String::new();
            for f in files.iter() {
                let inf = std::fs::read_to_string(&f)?;
                let out = inf.trim();
                if out.is_empty() {
                    continue;
                }
                outstr += out;
                if !outstr.ends_with('\n') {
                    outstr += "\n";
                }
            }

            illumos_image_builder::ensure::filestr(
                &outstr,
                &output_path,
                ROOT,
                ROOT,
                0o644,
                illumos_image_builder::ensure::Create::Always,
            )?;

            Ok(())
        }
        Action::Group(_group) => Err(BuildError::NotYetSupported(String::from("Group")).into()),
        Action::User(user, pw) => {
            /*
             * Read the shadow file:
             */
            let path = Path::new(root).join("etc/shadow");

            let orig = illumos_image_builder::ShadowFile::load(&path)?;
            let mut copy = orig.clone();

            if let Some(password) = pw {
                copy.password_set(&user, &password)?;
            }

            if orig == copy {
                info!("no change to shadow file; skipping write");
            } else {
                info!("updating shadow file");
                copy.write(&path)?;
                illumos_image_builder::ensure::perms(&path, ROOT, ROOT, 0o400)?;
            }

            Ok(())
        }
        Action::Symlink(link) => {
            let target_path = Path::new(root).join(&link.target);
            let link_path = Path::new(root).join(&link.link);

            let owner = if let Some(user) = link.owner {
                illumos_image_builder::translate_uid(&user)?
            } else {
                0
            };

            let group = if let Some(group) = link.group {
                illumos_image_builder::translate_gid(&group)?
            } else {
                0
            };

            illumos_image_builder::ensure::symlink(&link_path, &target_path, owner, group)?;

            Ok(())
        }
        Action::Dir(dir) => {
            let target_path = Path::new(root).join(dir.path.strip_prefix("/")?);

            let owner = if let Some(user) = dir.common.owner {
                illumos_image_builder::translate_uid(&user)?
            } else {
                0
            };

            let group = if let Some(group) = dir.common.group {
                illumos_image_builder::translate_gid(&group)?
            } else {
                0
            };

            let mode = if let Some(mode) = dir.common.mode {
                mode
            } else {
                0o750
            };

            illumos_image_builder::ensure::directory(&target_path, owner, group, mode)?;

            Ok(())
        }
        Action::File(file) => {
            let target_path = Path::new(root).join(file.path.strip_prefix("/")?);

            let owner = if let Some(user) = file.common.owner {
                illumos_image_builder::translate_uid(&user)?
            } else {
                0
            };

            let group = if let Some(group) = file.common.group {
                illumos_image_builder::translate_gid(&group)?
            } else {
                0
            };

            let mode = if let Some(mode) = file.common.mode {
                mode
            } else {
                0o750
            };

            if let Some(src) = file.src {
                if file.is_template {
                    let template = bundle.get_template_string(&src)?;

                    let mut tera = tera::Tera::new("")?;
                    let res = tera.render_str(&template, &Context::new())?;

                    illumos_image_builder::ensure::filestr(
                        &res,
                        &target_path,
                        owner,
                        group,
                        mode,
                        illumos_image_builder::ensure::Create::Always,
                    )?;
                } else {
                    let source_path = bundle.get_file(&src)?;

                    illumos_image_builder::ensure::file(
                        &source_path,
                        &target_path,
                        owner,
                        group,
                        mode,
                        illumos_image_builder::ensure::Create::Always,
                    )?;
                }
            } else if let Some(content) = file.content {
                if file.is_template {
                    let mut tera = tera::Tera::new("")?;
                    let res = tera.render_str(&content, &Context::new())?;

                    illumos_image_builder::ensure::filestr(
                        &res,
                        &target_path,
                        owner,
                        group,
                        mode,
                        illumos_image_builder::ensure::Create::Always,
                    )?;
                } else {
                    illumos_image_builder::ensure::filestr(
                        &content,
                        &target_path,
                        owner,
                        group,
                        mode,
                        illumos_image_builder::ensure::Create::Always,
                    )?;
                }
            } else {
                bail!("Either source or content must be defined")
            }

            Ok(())
        }
        Action::Perm(perm) => {
            let target_path = Path::new(root).join(perm.path.strip_prefix("/")?);

            let owner = if let Some(user) = perm.common.owner {
                illumos_image_builder::translate_uid(&user)?
            } else {
                0
            };

            let group = if let Some(group) = perm.common.group {
                illumos_image_builder::translate_gid(&group)?
            } else {
                0
            };

            let mode = if let Some(mode) = perm.common.mode {
                mode
            } else {
                return Err(
                    BuildError::NoModeSpecified(perm.path.to_string_lossy().to_string()).into(),
                );
            };

            illumos_image_builder::ensure::perms(&target_path, owner, group, mode)?;

            Ok(())
        }
        Action::Ips(ips_actions) => {
            for action in ips_actions.actions {
                run_ips_action(root, action)?;
            }
            Ok(())
        }
    }
}

pub fn run_ips_action(root: &str, action: IpsActions) -> Result<()> {
    info!("Running {}", action);
    match action {
        IpsActions::InitializeImage => {
            illumos_image_builder::pkg(&["image-create", "-F", "-z", root])
        }
        IpsActions::InstallPackages(pkgs) => illumos_image_builder::pkg_install(
            root,
            pkgs.packages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        ),
        IpsActions::InstallOptionals => {
            Err(BuildError::NotYetSupported(String::from("install optionals")).into())
        }
        IpsActions::SetProperty(ips_properties) => {
            for (prop_name, prop_value) in ips_properties.properties {
                illumos_image_builder::pkg(&["-R", root, "set-property", &prop_name, &prop_value])?;
            }

            Ok(())
        }
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

            illumos_image_builder::pkg(
                args.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
        }
        IpsActions::ApprovePublisherCA(_) => {
            Err(BuildError::NotYetSupported(String::from("approving CA")).into())
        }
        IpsActions::UninstallPackages(pkgs) => illumos_image_builder::pkg_uninstall(
            root,
            pkgs.packages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        ),
        IpsActions::SetVariant(variant_props) => {
            for (variant_name, variant_value) in variant_props.properties {
                illumos_image_builder::pkg_ensure_variant(root, &variant_name, &variant_value)?;
            }

            Ok(())
        }
        IpsActions::SetFacet(facet_prop) => {
            for (facet_name, facet_value) in facet_prop.properties {
                illumos_image_builder::pkg_ensure_facet(root, &facet_name, &facet_value)?;
            }

            Ok(())
        }
        IpsActions::PurgeHistory => illumos_image_builder::pkg(&["-R", root, "purge-history"]),
        IpsActions::SetMediator(mediator_props) => {
            let mut args = vec!["-R".to_owned(), root.to_string(), "set-mediator".to_owned()];
            if let Some(imple) = mediator_props.implementation {
                args.push("-I".to_owned());
                args.push(imple);
            }

            if let Some(vers) = mediator_props.version {
                args.push("-V".to_owned());
                args.push(vers);
            }

            args.push(mediator_props.name);

            illumos_image_builder::pkg(
                args.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
        }
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
