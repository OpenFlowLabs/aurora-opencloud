pub mod rpc;

use miette::Diagnostic;
use osshkeys::keys::FingerprintHash;
use osshkeys::PublicParts;
use pasetors::claims::Claims;
use pasetors::footer::Footer;
use pasetors::keys::{AsymmetricPublicKey, AsymmetricSecretKey};
use pasetors::version4::V4;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tonic::metadata::errors::InvalidMetadataValue;

pub static AUTH_FILE_LOCATION: &str = "opc/principals.yaml";
pub static CONFIG_FILE_LOCATION: &str = "opc/config.yaml";
pub static CURRENT_AUTH_ENTRY_KEY: &str = "principals.current";
pub static AUTHORIZATION_HEADER: &str = "authorization";

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Paseto(#[from] pasetors::errors::Error),

    #[error(transparent)]
    OSSH(#[from] osshkeys::error::Error),

    #[error(transparent)]
    UTF8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Ossl(#[from] openssl::error::ErrorStack),

    #[error("Only Ed25519 Keys are supported for now")]
    UnsupportedSSHKey,

    #[error("no auth entry named {0}")]
    NoSuchAuthEntry(String),

    #[error(transparent)]
    InvalidMetadata(#[from] InvalidMetadataValue),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),

    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    #[error("config key: {0} can not be set")]
    UnsetableConfig(String),
}

pub type Result<T> = miette::Result<T, Error>;

pub fn get_auth_config_location() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join(AUTH_FILE_LOCATION)
    } else {
        Path::new(&format!(".config/{}", AUTH_FILE_LOCATION)).into()
    }
}

pub fn get_config_location() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join(CONFIG_FILE_LOCATION)
    } else {
        Path::new(&format!(".config/{}", CONFIG_FILE_LOCATION)).into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ClientConfig {
    pub api: APIDestination,
    pub principals: PrincipalConfig,
}

#[derive(Deserialize, Serialize)]
pub struct PrincipalConfig {
    pub current: String,
}

#[derive(Deserialize, Serialize)]
pub struct APIDestination {
    pub host: String,
    pub port: String,
    pub secure_connection: bool,
}

impl APIDestination {
    pub fn get_uri(&self) -> String {
        if self.secure_connection {
            format!("https://{}:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        }
    }
}

pub fn read_config(
    host: Option<String>,
    port: Option<String>,
    ssl: Option<bool>,
) -> Result<ClientConfig> {
    let config_file_path = get_config_location().to_string_lossy().to_string();
    let cfg = config::Config::builder()
        .set_default("api.host", "127.0.0.1")?
        .set_default("api.port", "50051")?
        .set_default("api.secure_connection", false)?
        .set_default(CURRENT_AUTH_ENTRY_KEY, "")?
        .add_source(config::File::with_name(&config_file_path).required(false))
        .set_override_option("api.host", host)?
        .set_override_option("api.port", port)?
        .set_override_option("api.secure_connection", ssl)?
        .build()?;

    let cc = cfg.try_deserialize()?;
    Ok(cc)
}

pub fn write_config(config_struct: ClientConfig) -> Result<()> {
    let config_file_path = get_config_location();
    if !config_file_path.exists() {
        let mut config_dir = config_file_path.clone();
        config_dir.pop();
        std::fs::create_dir_all(config_dir)?;
    }
    let mut config_file = File::create(config_file_path)?;
    serde_yaml::to_writer(&mut config_file, &config_struct)?;
    Ok(())
}

pub fn read_auth_config() -> Result<Vec<AuthEntry>> {
    let auth_file_path = get_auth_config_location();
    if !auth_file_path.exists() {
        return Ok(vec![]);
    }
    let auth_file = File::open(auth_file_path)?;
    let auth_struct: Vec<AuthEntry> = serde_yaml::from_reader(&auth_file)?;
    Ok(auth_struct)
}

pub fn write_auth_config(auth_struct: Vec<AuthEntry>) -> Result<()> {
    let auth_file_path = get_auth_config_location();
    if !auth_file_path.exists() {
        let mut auth_dir = auth_file_path.clone();
        auth_dir.pop();
        std::fs::create_dir_all(auth_dir)?;
    }
    let mut auth_file = File::create(auth_file_path)?;
    serde_yaml::to_writer(&mut auth_file, &auth_struct)?;
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthEntry {
    pub name: String,
    pub key_location: String,
    pub pk_fingerprint: String,
    pub passphrase: bool,
}

pub fn get_auth_entry<'a>(name: &str, entries: &'a [AuthEntry]) -> Result<&'a AuthEntry> {
    for entry in entries {
        if entry.name == name {
            return Ok(entry);
        }
    }
    Err(Error::NoSuchAuthEntry(name.to_owned()))
}

pub struct KeyPairWithFingerprint {
    pub fingerprint: String,
    pub secret: AsymmetricSecretKey<V4>,
    pub public: AsymmetricPublicKey<V4>,
}

pub fn read_key<P: AsRef<Path>>(
    key_path: P,
    passphrase: Option<&str>,
) -> Result<KeyPairWithFingerprint> {
    let secret_key_str = std::fs::read_to_string(key_path.as_ref())?;
    let ossl_key = if let Some(passphrase) = passphrase {
        openssl::pkey::PKey::private_key_from_pem_passphrase(
            secret_key_str.as_bytes(),
            passphrase.as_bytes(),
        )
    } else {
        openssl::pkey::PKey::private_key_from_pem(secret_key_str.as_bytes())
    }?;

    if ossl_key.id() != openssl::pkey::Id::ED25519 {
        return Err(Error::UnsupportedSSHKey);
    }

    let secret_key = pasetors::keys::AsymmetricSecretKey::<V4>::from(&ossl_key.raw_private_key()?)?;
    let public_key = pasetors::keys::AsymmetricPublicKey::<V4>::from(&ossl_key.raw_public_key()?)?;
    let ossh_public_key_string = String::from_utf8(ossl_key.public_key_to_pem()?)?;
    let ossh_public_key = osshkeys::PublicKey::from_keystr(&ossh_public_key_string)?;

    let fingerprint = hex::encode(ossh_public_key.fingerprint(FingerprintHash::SHA256)?);

    Ok(KeyPairWithFingerprint {
        fingerprint,
        public: public_key,
        secret: secret_key,
    })
}

pub fn make_token_for_auth_entry(entry: &AuthEntry) -> Result<String> {
    let mut claims = Claims::new()?;
    claims.subject(&entry.name)?;
    let mut footer = Footer::new();
    footer.add_additional("fingerprint", &entry.pk_fingerprint)?;
    let keypair = if entry.passphrase {
        //TODO: Ask user for passphrase
        read_key(&entry.key_location, None)
    } else {
        read_key(&entry.key_location, Some("dummy"))
    }?;

    let token = pasetors::public::sign(
        &keypair.secret,
        &keypair.public,
        &claims,
        Some(&footer),
        None,
    )?;
    Ok(token)
}
