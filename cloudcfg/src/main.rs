extern crate common;

mod rpc;

use std::path::{PathBuf, Path};
use std::fs::File;
use std::collections::HashMap;
use pasetors::footer::Footer;
use serde::{Serialize, Deserialize};
use common::*;
use rpc::tenant::tenant_client::TenantClient;
use rpc::tenant::PingMsg;
use clap::{Parser, Subcommand};
use prettytable::{Table, cell, row};
use pasetors::claims::{Claims};
use pasetors::version4::V4;
use pasetors::keys::{AsymmetricPublicKey, AsymmetricSecretKey};
use osshkeys::{PublicParts};
use osshkeys::keys::FingerprintHash;

static AUTH_FILE_LOCATION: &str = "aurora-opencloud/principals.yaml";
static CONFIG_FILE_LOCATION: &str = "aurora-opencloud/config.yaml";
static CURRENT_AUTH_ENTRY_KEY: &str = "principals.current";

fn get_auth_config_location() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join(AUTH_FILE_LOCATION)
    } else {
        Path::new(&format!(".config/{}", AUTH_FILE_LOCATION)).into()
    }
}

fn get_config_location() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join(CONFIG_FILE_LOCATION)
    } else {
        Path::new(&format!(".config/{}", CONFIG_FILE_LOCATION)).into()
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(long, short, default_value="127.0.0.1")]
    host: String,

    #[clap(long, short, default_value="50051")]
    port: String,

    #[clap(long, short, default_value="false")]
    secure_connection: bool,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config {
        #[clap(value_parser)]
        key: Option<String>,

        #[clap(value_parser)]
        value: Option<String>,
    },
    Auth {
        #[clap(subcommand)]
        cmd: AuthCommands,
    },
    Ping,
}

#[derive(Subcommand, Debug)]
enum AuthCommands {
    List,
    Add {
        #[clap(value_parser)]
        name: String,

        #[clap(value_parser)]
        key_location: String,

        #[clap(short='p', long, action)]
        passphrase: bool,
    },
    Remove {
        #[clap(value_parser)]
        name: String,
    }
}

impl Cli {
    fn get_api_destination(&self) -> String {
        if self.secure_connection {
            format!("https://{}:{}", self.host, self.port)
        } else {
            format!("http://{}:{}", self.host, self.port)
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AuthEntry {
    name: String,
    key_location: String,
    pk_fingerprint: String,
    passphrase: bool
}

fn read_config() -> Result<HashMap<String,String>> {
    let config_file_path = get_config_location();
    if !config_file_path.exists() {
        return Ok(HashMap::new());
    }
    let config_file = File::open(config_file_path)?;
    let config_struct: HashMap<String, String> = serde_yaml::from_reader(&config_file)?;
    Ok(config_struct)
}

fn write_config(config_struct: HashMap<String,String>) -> Result<()> {
    let config_file_path = get_config_location();
    if !config_file_path.exists() {
        let mut config_dir = config_file_path.clone();
        config_dir.pop();
        std::fs::create_dir_all(config_dir)?;
    }
    let mut config_file = if !config_file_path.exists() {
        File::create(config_file_path)
    } else {
        File::open(config_file_path)
    }?;
    serde_yaml::to_writer(&mut config_file, &config_struct)?;
    Ok(())
}

fn read_auth_config() -> Result<Vec<AuthEntry>> {
    let auth_file_path = get_auth_config_location();
    if !auth_file_path.exists() {
        return Ok(vec![]);
    }
    let auth_file = File::open(auth_file_path)?;
    let auth_struct: Vec<AuthEntry> = serde_yaml::from_reader(&auth_file)?;
    Ok(auth_struct)
}

fn write_auth_config(auth_struct: Vec<AuthEntry>) -> Result<()> {
    let auth_file_path = get_auth_config_location();
    if !auth_file_path.exists() {
        let mut auth_dir = auth_file_path.clone();
        auth_dir.pop();
        std::fs::create_dir_all(auth_dir)?;
    }
    let mut auth_file = if !auth_file_path.exists() {
        File::create(auth_file_path)
    } else {
        File::open(auth_file_path)
    }?;
    serde_yaml::to_writer(&mut auth_file, &auth_struct)?;
    Ok(())
}

fn get_auth_entry<'a>(name: &str, entries: &'a [AuthEntry]) -> Result<&'a AuthEntry> {
    for entry in entries {
        if entry.name == name {
            return Ok(entry)
        }
    }
    bail!("no entry named {}", name)
}

struct KeyPairWithFingerprint {
    fingerprint: String,
    secret: AsymmetricSecretKey<V4>,
    public: AsymmetricPublicKey<V4>
}

fn read_key<P: AsRef<Path>>(key_path: P, passphrase: Option<&str>) -> Result<KeyPairWithFingerprint> {
    let secret_key_str = std::fs::read_to_string(key_path.as_ref())?;
    let ossl_key = if let Some(passphrase) = passphrase {
        openssl::pkey::PKey::private_key_from_pem_passphrase(secret_key_str.as_bytes(), passphrase.as_bytes())
    } else {
        openssl::pkey::PKey::private_key_from_pem(secret_key_str.as_bytes())
    }?;

    if ossl_key.id() != openssl::pkey::Id::ED25519 {
        bail!("Only Ed25519 Keys are supported for now")
    }

    let secret_key = pasetors::keys::AsymmetricSecretKey::<V4>::from(&ossl_key.raw_private_key()?)?;
    let public_key = pasetors::keys::AsymmetricPublicKey::<V4>::from(&ossl_key.raw_public_key()?)?;
    let ossh_public_key_string = String::from_utf8(ossl_key.public_key_to_pem()?)?;
    let ossh_public_key = osshkeys::PublicKey::from_keystr(&ossh_public_key_string)?;
    
    let fingerprint = hex::encode(ossh_public_key.fingerprint(FingerprintHash::SHA256)?);

    Ok(KeyPairWithFingerprint{ 
        fingerprint,
        public: public_key, 
        secret: secret_key, 
    })
}

fn make_token_for_auth_entry(entry: &AuthEntry) -> Result<String> {
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
        &keypair.secret, &keypair.public, 
        &claims, Some(&footer), None)?;
    Ok(token)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _guard = init_slog_logging(false)?;
    let mut client = TenantClient::connect(cli.get_api_destination()).await?;

    match cli.command {
        Commands::Config { key, value } => {
            let do_modify = if key.is_some() {
                value.is_some()
            } else {
                false
            };

            let do_remove = value.is_none();

            let mut config_struct = read_config()?;

            if !do_modify {
                let mut table = Table::new();

                table.add_row(row!["KEY", "VALUE"]);
                for (key, value) in config_struct {
                    table.add_row(row![key, value]);
                }

                // Print the table to stdout
                table.printstd();
            } else {
                if do_remove {
                    config_struct.remove(&key.unwrap());
                } else {
                    config_struct.insert(key.unwrap(), value.unwrap());
                }

                write_config(config_struct)?;
            }
        },
        Commands::Auth { cmd } => {
            let mut auth_struct = read_auth_config()?;
            match cmd {
                AuthCommands::List => {
                    let mut table = Table::new();

                    // Add a row per time
                    table.add_row(row!["NAME", "FINGERPRINT", "KEYFILE", "HAS PASSPHRASE"]);
                    for entry in auth_struct {
                        table.add_row(row![
                            entry.name,
                            entry.pk_fingerprint,
                            entry.key_location, 
                            entry.passphrase,
                        ]);
                    }

                    // Print the table to stdout
                    table.printstd();
                },
                AuthCommands::Add { name, key_location, passphrase } => {
                    //TODO: read passphrase from console
                    let keypair = read_key(&key_location, None)?;

                    auth_struct.push(AuthEntry { 
                        name: name.clone(),
                        pk_fingerprint: keypair.fingerprint,
                        key_location, 
                        passphrase 
                    });

                    info!("Writing auth config");
                    write_auth_config(auth_struct)?;
                    let mut config: HashMap<String, String> = HashMap::new();
                    config.insert(CURRENT_AUTH_ENTRY_KEY.to_owned(), name);
                    write_config(config)?;
                },
                AuthCommands::Remove { name } => {
                    let auth_struct: Vec<AuthEntry> = auth_struct.into_iter().filter(|entry| {
                        entry.name != name
                    }).collect();
                    write_auth_config(auth_struct)?;
                },
            }
        },
        Commands::Ping => {
            let mut request = tonic::Request::new(PingMsg {
                sender: "cloudcfg".into(),
            });

            let config = read_config();
            let auth = read_auth_config();
            if config.is_ok() && auth.is_ok() {
                let config = config.unwrap();
                let auth = auth.unwrap();
                let entry = get_auth_entry(&config[CURRENT_AUTH_ENTRY_KEY], &auth)?;
                let token = make_token_for_auth_entry(entry)?;
                request.metadata_mut().insert(AUTHORIZATION_HEADER, token.parse()?);
            }

            let response = client.ping(request).await?;

            info!("Pong Status: {}", response.into_inner().pong);
        },
    }

    Ok(())
}
