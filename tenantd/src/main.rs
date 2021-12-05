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
use rpc::tenant_server::{Tenant, TenantServer};
use rpc::{PingMsg, PongMsg};

#[derive(Debug, Default)]
pub struct TenantSvc {}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let logger = init_log();
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = set_global_logger(logger);

    let tenant_service = TenantSvc::default();
    info!("Starting Tenant Service");
    Server::builder()
        .add_service(TenantServer::new(tenant_service))
        .serve(addr)
        .await?;

    Ok(())
}
