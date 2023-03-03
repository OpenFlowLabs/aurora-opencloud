use hard_xml::{XmlRead, XmlWrite};
use miette::Diagnostic;
use std::{fmt::Display, path::Path, str::FromStr};
use thiserror::Error;

type Result<T> = miette::Result<T, SMFError>;

const SITE_MANIFEST_HEADER: &str = r#"<?xml version='1.0'?>
<!DOCTYPE service_bundle SYSTEM "/usr/share/lib/xml/dtd/service_bundle.dtd.1">"#;

#[derive(Debug, Error, Diagnostic)]
pub enum SMFError {
    #[error("could not parse {0} as {1}")]
    ParseError(String, String),
    #[error(transparent)]
    XMLError(#[from] hard_xml::XmlError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BundleType {
    Profile,
}

impl Default for BundleType {
    fn default() -> Self {
        Self::Profile
    }
}

impl FromStr for BundleType {
    type Err = SMFError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "profile" => Ok(Self::Profile),
            x => Err(SMFError::ParseError(x.to_owned(), String::from("profile"))),
        }
    }
}

impl Display for BundleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleType::Profile => write!(f, "profile"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ServiceType {
    Service,
}

impl Default for ServiceType {
    fn default() -> Self {
        Self::Service
    }
}

impl FromStr for ServiceType {
    type Err = SMFError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "service" => Ok(Self::Service),
            x => Err(SMFError::ParseError(x.to_owned(), String::from("service"))),
        }
    }
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Service => write!(f, "service"),
        }
    }
}

#[derive(Debug, PartialEq, XmlRead, XmlWrite, Default, Clone)]
#[xml(tag = "service_bundle")]
pub struct ServiceBundle {
    #[xml(attr = "type")]
    pub bundle_type: BundleType,
    #[xml(attr = "name")]
    pub name: String,
    #[xml(child = "service")]
    pub services: Vec<Service>,
}

#[derive(Debug, PartialEq, XmlRead, XmlWrite, Default, Clone)]
#[xml(tag = "service")]
pub struct Service {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "version")]
    pub version: i32,
    #[xml(attr = "type")]
    pub service_type: ServiceType,
    #[xml(child = "instance")]
    pub instances: Vec<Instance>,
}

#[derive(Debug, PartialEq, XmlRead, XmlWrite, Default, Clone)]
#[xml(tag = "instance")]
pub struct Instance {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "enabled")]
    pub enabled: Option<bool>,
    #[xml(child = "property_group")]
    pub property_groups: Vec<PropertyGroup>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PropertyGroupType {
    System,
}

impl Default for PropertyGroupType {
    fn default() -> Self {
        Self::System
    }
}

impl FromStr for PropertyGroupType {
    type Err = SMFError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "system" => Ok(Self::System),
            x => Err(SMFError::ParseError(
                x.to_owned(),
                String::from("property group"),
            )),
        }
    }
}

impl Display for PropertyGroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyGroupType::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, PartialEq, XmlRead, XmlWrite, Default, Clone)]
#[xml(tag = "property_group")]
pub struct PropertyGroup {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "type")]
    pub pg_type: PropertyGroupType,
    #[xml(child = "propval")]
    pub values: Vec<PropVal>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PropValType {
    AString,
}

impl Default for PropValType {
    fn default() -> Self {
        Self::AString
    }
}

impl FromStr for PropValType {
    type Err = SMFError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "astring" | "string" => Ok(Self::AString),
            x => Err(SMFError::ParseError(
                x.to_owned(),
                String::from("propval type"),
            )),
        }
    }
}

impl Display for PropValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropValType::AString => write!(f, "astring"),
        }
    }
}

#[derive(Debug, PartialEq, XmlRead, XmlWrite, Default, Clone)]
#[xml(tag = "propval")]
pub struct PropVal {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "type")]
    pub pb_type: PropValType,
    #[xml(attr = "value")]
    pub value: String,
}

pub fn parse_site_manifest<P: AsRef<Path>>(path: P) -> Result<ServiceBundle> {
    use std::fs::read_to_string;
    let content = read_to_string(path.as_ref())?;
    Ok(ServiceBundle::from_str(&content)?)
}

pub fn write_site_manifest<P: AsRef<Path>>(path: P, bundle: &ServiceBundle) -> Result<()> {
    use std::fs::write;
    let content = vec![
        SITE_MANIFEST_HEADER.clone().to_string(),
        bundle.to_string()?,
    ]
    .join("\n");
    write(path.as_ref(), &content.into_bytes())?;
    Ok(())
}

pub fn site_manifest_to_string(bundle: &ServiceBundle) -> Result<String> {
    let content = vec![
        SITE_MANIFEST_HEADER.clone().to_string(),
        bundle.to_string()?,
    ];
    Ok(content.join("\n"))
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use miette::{IntoDiagnostic, Result};

    use crate::smf::{PropValType, PropertyGroupType, Service};

    use super::{
        site_manifest_to_string, BundleType, Instance, PropVal, PropertyGroup, ServiceType,
    };

    use super::ServiceBundle;

    use hard_xml::XmlRead;

    #[test]
    fn test_site_xml_parse() -> Result<()> {
        let expected = ServiceBundle {
            bundle_type: BundleType::Profile,
            name: "sc_install_interactive".into(),
            services: [
                Service {
                    name: "system/keymap".into(),
                    version: 1,
                    service_type: ServiceType::Service,
                    instances: [Instance {
                        name: "default".into(),
                        enabled: None,
                        property_groups: [PropertyGroup {
                            name: "keymap".into(),
                            pg_type: PropertyGroupType::System,
                            values: [PropVal {
                                name: "layout".into(),
                                pb_type: PropValType::AString,
                                value: "US-English".into(),
                            }]
                            .to_vec(),
                        }]
                        .to_vec(),
                    }]
                    .to_vec(),
                },
                Service {
                    name: "application/graphical-login/gdm".into(),
                    version: 1,
                    service_type: ServiceType::Service,
                    instances: [Instance {
                        name: "default".into(),
                        enabled: Some(true),
                        property_groups: [].to_vec(),
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
        };
        let content = read_to_string("testdata/site.xml").into_diagnostic()?;
        let actual = ServiceBundle::from_str(&content).into_diagnostic()?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_full_cycle() -> Result<()> {
        let content = read_to_string("testdata/site.xml").into_diagnostic()?;
        let expected = content
            .split("\n")
            .into_iter()
            .map(|line| line.trim().to_owned())
            .collect::<Vec<String>>()
            .join("")
            .replace("<!--br-->", "\n");
        let test = ServiceBundle::from_str(&expected).into_diagnostic()?;
        let actual = site_manifest_to_string(&test)?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
