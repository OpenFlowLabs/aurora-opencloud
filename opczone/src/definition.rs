use std::{collections::HashMap, path::PathBuf};

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
