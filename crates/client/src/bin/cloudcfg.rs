use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cloudcfg::rpc::tenant::tenant_client::TenantClient;
use cloudcfg::rpc::tenant::PingMsg;
use cloudcfg::*;
use prettytable::{row, Table};
use tracing::{info, trace};
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[arg(long, short = 'H')]
    host: Option<String>,

    #[arg(long, short)]
    port: Option<String>,

    #[arg(long, short)]
    secure_connection: Option<bool>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config {
        key: String,

        value: Option<String>,
    },
    Auth {
        #[command(subcommand)]
        cmd: AuthCommands,
    },
    Ping,
    Connect {
        uri: url::Url,

        invite_token: Option<String>,
    },
    Nodes {
        #[command(subcommand)]
        cmd: NodeCommands,
    },
}

#[derive(Subcommand, Debug)]
enum AuthCommands {
    List,
    Add {
        name: String,

        key_location: String,

        #[arg(short = 'c', long = "current", default_value = "false")]
        set_as_current: bool,

        #[arg(short = 'p', long, action)]
        passphrase: bool,
    },
    Remove {
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum NodeCommands {
    List,
    Define { file: PathBuf },
    Delete { id: String },
    Update { id: String, file: PathBuf },
    Show { id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { key, value } => {
            let mut cfg = read_config(cli.host, cli.port, cli.secure_connection)?;

            match key.as_str() {
                "host" => {
                    if let Some(val) = value {
                        cfg.api.host = val;
                    } else {
                        cfg.api.host = String::new();
                    }
                }
                "port" => {
                    if let Some(port) = value {
                        cfg.api.port = port;
                    } else {
                        cfg.api.port = String::new();
                    }
                }
                unknown_key => {
                    return Err(Error::UnsetableConfig(unknown_key.to_owned()));
                }
            }

            write_config(cfg)?;
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
                    set_as_current,
                } => {
                    //TODO: read passphrase from console
                    let keypair = read_key(&key_location, None)?;

                    auth_struct.push(AuthEntry {
                        name: name.clone(),
                        pk_fingerprint: keypair.fingerprint,
                        key_location,
                        passphrase,
                    });

                    trace!("Writing Auth config");
                    write_auth_config(auth_struct)?;
                    if set_as_current {
                        let mut cfg = read_config(None, None, None)?;
                        trace!("Setting principal name {} as current", &name);
                        cfg.principals.current = name;
                        write_config(cfg)?;
                    }
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
            let cfg = read_config(cli.host, cli.port, cli.secure_connection)?;

            let mut client = TenantClient::connect(cfg.api.get_uri()).await?;
            let mut request = tonic::Request::new(PingMsg {
                sender: "cloudcfg".into(),
            });
            let auth = read_auth_config()?;

            let entry = get_auth_entry(&cfg.principals.current, &auth)?;
            let token = make_token_for_auth_entry(entry)?;
            request
                .metadata_mut()
                .insert(AUTHORIZATION_HEADER, token.parse()?);

            let response = client.ping(request).await?;

            info!("Pong Status: {}", response.into_inner().pong);
        }
    }

    Ok(())
}
