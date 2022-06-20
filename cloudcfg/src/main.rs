extern crate common;

mod rpc;

use common::*;
use rpc::tenant::tenant_client::TenantClient;
use rpc::tenant::PingMsg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = TenantClient::connect("http://127.0.0.1:50051").await?;
    let _guard = init_slog_logging(false)?;
    let request = tonic::Request::new(PingMsg {
        sender: "cloudcfg".into(),
    });

    let response = client.ping(request).await?;

    info!("Pong Status: {}", response.into_inner().pong);

    Ok(())
}
