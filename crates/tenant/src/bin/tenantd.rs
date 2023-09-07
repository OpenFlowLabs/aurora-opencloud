use biscuit_auth::PrivateKey;
use clap::{Parser, Subcommand};
use miette::IntoDiagnostic;
use tenant::*;
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Debug, Parser)]
struct Args {
    /// Path of the configuration file if it is not under /etc/opc/tenantd.yaml
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a configuration with keys
    Init,
    /// Start serving requests
    Serve { listen: Option<String> },
    /// Create a token used for when you don't have an account yet
    CreateInitToken,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().pretty())
        .try_init()
        .into_diagnostic()?;

    match args.cmd {
        Commands::Init => {
            init_config(args.config)?;
        }
        Commands::Serve {
            listen: listen_addr,
        } => {
            let mut cfg = read_config(args.config).into_diagnostic()?;
            if let Some(addr) = listen_addr {
                cfg.set_listen_addr(addr);
            }

            debug!(?cfg, "Starting Tenant Service");
            listen(cfg).await.into_diagnostic()?;
        }
        Commands::CreateInitToken => {
            let cfg = read_config(args.config).into_diagnostic()?;

            debug!("Generating Initialization token");
            let token = generate_init_token(&cfg).into_diagnostic()?;
            println!("Use this token to initialize the root tenant:");
            println!("{}", token.to_base64().into_diagnostic()?);
        }
    }

    Ok(())
}
