//#[macro_use]
extern crate common;
extern crate diesel;
extern crate dotenv;
extern crate uuid;

extern crate chrono;

use tonic::{transport::Server, Request, Response, Status};

mod rpc {
    tonic::include_proto!("tenant"); // The string specified here must match the proto package name
}

use common::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use josekit::jws::EdDSA;
use rpc::tenant_server::{Tenant, TenantServer};
use rpc::{PingMsg, PongMsg};
use std::env;
use std::sync::Arc;

type PGPool = Arc<Pool<ConnectionManager<PgConnection>>>;

pub struct TenantSvc {
    pool: PGPool,
}

#[tonic::async_trait]
impl Tenant for TenantSvc {
    async fn ping(&self, request: Request<PingMsg>) -> Result<Response<PongMsg>, Status> {
        // Log the ping we have gotten so we see some trafick
        info!("received ping from: {}", request.into_inner().sender);

        let reply = rpc::PongMsg {
            status: rpc::pong_msg::Status::Success.into(),
        };

        Ok(Response::new(reply))
    }
}

impl TenantSvc {
    pub fn new(pool: PGPool) -> Self {
        TenantSvc { pool }
    }
}

// TODO: Embed migrations
// https://docs.diesel.rs/1.4.x/diesel_migrations/macro.embed_migrations.html

// TODO: use clap for the daemon and initialize subcommands

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let logger = init_log();
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = set_global_logger(logger);

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let (pub_key_path, priv_key_path) = {
        let keys_path = env::var("KEY_DIRECTORY").unwrap_or(String::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/sample_data/keys"
        )));
        (
            keys_path.clone() + "/ED25519_public.pem",
            keys_path + "/ED25519_private.pem",
        )
    };

    let issuer_base = env::var("ISSUER_BASE").expect("ISSUER_BASE must be set");

    let private_key = std::fs::read(priv_key_path.clone())
        .expect(format!("could not find private_key for JWT in {}", &priv_key_path).as_str());
    let public_key = std::fs::read(pub_key_path.clone())
        .expect(format!("could not find public_key for JWT in {}", &pub_key_path).as_str());
    let signer = EdDSA
        .signer_from_pem(&private_key)
        .expect(format!("cannot make signer from private_key is it PKCS#8 formatted?").as_str());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(Pool::builder().max_size(15).build(manager).unwrap());

    let tenant_service = TenantSvc::new(pool);
    info!("Starting Tenant Service");
    Server::builder()
        .add_service(TenantServer::new(tenant_service))
        .serve(addr)
        .await?;

    Ok(())
}
