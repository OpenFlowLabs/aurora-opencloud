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
    CreateInitToken {
        // Optional limit token to tenant
        // if you enter a name here the token will be limited to create a tenant with the name only
        // and allow the user to create a Principal
        tenant: Option<String>,
    },
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
        Commands::CreateInitToken { tenant } => {}
    }

    Ok(())
}
