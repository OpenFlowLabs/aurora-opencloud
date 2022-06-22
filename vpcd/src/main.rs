use std::{env, net::SocketAddr, sync::Arc};

mod vpc;
mod database;

use common::*;
use clap::{Parser, Subcommand};
use diesel::{r2d2::{ConnectionManager, Pool}, PgConnection};
use dotenv::{dotenv};
use vpc::vpc_server::{Vpc};
use tonic::{transport::Server, Request, Response, Status};
use database::PGPool;

use crate::vpc::vpc_server::VpcServer;

pub struct VPCSvc {
    pool: PGPool,
}

#[tonic::async_trait]
impl Vpc for VPCSvc {
    async fn list_vpcs(&self, request: Request<vpc::ListVpcRequest>) -> Result<Response<vpc::ListVpcResponse>, Status> {
        Err(Status::aborted("unimplemented"))
    }

    async fn create_vpc(&self, request: Request<vpc::CreateVpcRequest>) -> Result<Response<vpc::StatusResponse>, Status> {
        Err(Status::aborted("unimplemented"))
    }

    async fn get_vpc(&self, request: Request<vpc::GetVpcRequest>) -> Result<Response<vpc::VpcSchema>, Status> {
        Err(Status::aborted("unimplemented"))
    }
}

impl VPCSvc {
    pub fn new(pool: PGPool) -> Result<Self> {
        Ok(VPCSvc {
            pool,
        })
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: CliCommands,

    #[clap(long, short, env, value_parser)]
    database_url: Option<String>,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    Serve {
        #[clap(long, short, default_value_t = String::from("127.0.0.1"), value_parser)]
        listen: String,

        #[clap(long, short, default_value_t = String::from("50051"), value_parser)]
        port: String,
    },
}

async fn serve(
    listen: &str,
    port: &str,
    database_url: &str,
) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", listen, port).parse()?;

    let pool = build_database_connection(database_url)?;

    let vpc_service = VPCSvc::new(
        pool,
    )?;

    info!("Starting Tenant Service");
    Server::builder()
        .add_service(VpcServer::new(vpc_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn build_database_connection(database_url: &str) -> Result<PGPool> {
    info!("Initiating Database connection");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(Pool::builder().max_size(15).build(manager)?);
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _guard = init_slog_logging(false)?;

    dotenv().ok();

    let database_url = if let Some(database_url) = cli.database_url {
        database_url
    } else {
        env::var("DATABASE_URL")?
    };

    match cli.command {
        CliCommands::Serve { listen, port } => {
            serve(&listen, &port, &database_url).await
        },
    }
}
