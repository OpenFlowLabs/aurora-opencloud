extern crate common;

mod rpc {
    tonic::include_proto!("tenant");
}

use common::*;
use rpc::tenant_client::TenantClient;
use rpc::PingMsg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TenantClient::connect("http://127.0.0.1:50051").await?;
    let logger = init_log();
    let _guard = set_global_logger(logger);
    let request = tonic::Request::new(PingMsg {
        sender: "cloudcfg".into(),
    });

    let response = client.ping(request).await?;

    info!("Pong Status: {}", response.into_inner().status);

    Ok(())
}
