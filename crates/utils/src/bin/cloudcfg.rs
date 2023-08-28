use std::collections::HashMap;

use clap::{Parser, Subcommand};
use cloudcfg::rpc::tenant::tenant_client::TenantClient;
use cloudcfg::rpc::tenant::PingMsg;
use cloudcfg::*;
use prettytable::{cell, row, Table};
use tracing::info;
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(long, short, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, short, default_value = "50051")]
    port: String,

    #[clap(long, short, default_value = "false")]
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

        #[clap(short = 'p', long, action)]
        passphrase: bool,
    },
    Remove {
        #[clap(value_parser)]
        name: String,
    },
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "oxiblog=trace,tower_http=trace,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cli = Cli::parse();

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
        }
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
                }
                AuthCommands::Add {
                    name,
                    key_location,
                    passphrase,
                } => {
                    //TODO: read passphrase from console
                    let keypair = read_key(&key_location, None)?;

                    auth_struct.push(AuthEntry {
                        name: name.clone(),
                        pk_fingerprint: keypair.fingerprint,
                        key_location,
                        passphrase,
                    });

                    info!("Writing auth config");
                    write_auth_config(auth_struct)?;
                    let mut config: HashMap<String, String> = HashMap::new();
                    config.insert(CURRENT_AUTH_ENTRY_KEY.to_owned(), name);
                    write_config(config)?;
                }
                AuthCommands::Remove { name } => {
                    let auth_struct: Vec<AuthEntry> = auth_struct
                        .into_iter()
                        .filter(|entry| entry.name != name)
                        .collect();
                    write_auth_config(auth_struct)?;
                }
            }
        }
        Commands::Ping => {
            let mut request = tonic::Request::new(PingMsg {
                sender: "cloudcfg".into(),
            });

            let config = read_config();
            let auth = read_auth_config();
            if let Ok(config) = config {
                if let Ok(auth) = auth {
                    let entry = get_auth_entry(&config[CURRENT_AUTH_ENTRY_KEY], &auth)?;
                    let token = make_token_for_auth_entry(entry)?;
                    request
                        .metadata_mut()
                        .insert(AUTHORIZATION_HEADER, token.parse()?);
                }
            }

            let response = client.ping(request).await?;

            info!("Pong Status: {}", response.into_inner().pong);
        }
    }

    Ok(())
}
