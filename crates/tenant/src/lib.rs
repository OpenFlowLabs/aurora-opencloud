mod database;
mod migrations;
mod rpc;

use biscuit_auth::{KeyPair, PrivateKey, RootKeyProvider};
use config::builder::DefaultState;
use miette::Diagnostic;
use migrations::{Migrator, MigratorTrait};
use rpc::tenant::{
    tenant_server::{Tenant, TenantServer},
    AddPublicKeyRequest, AttributeRequest, CreatePrincipalRequest, CreateTenantRequest,
    DeletePrincipalRequest, DeleteTenantRequest, GetPrincipalRequest, GetTenantRequest,
    ListPrincipalRequest, ListPrincipalResponse, ListTenantRequest, ListTenantResponse, PingMsg,
    PongMsg, PrincipalResponse, PublicKeyResponse, RemovePublicKeyRequest, RoleRequest,
    StatusResponse, TenantResponse,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::fs::File;
use thiserror::Error;
use tonic::transport::Server;
use tracing::{debug, instrument};

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

    #[error(transparent)]
    Config(#[from] config::ConfigError),

    #[error(transparent)]
    BiscuitFormat(#[from] biscuit_auth::error::Format),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    SeaMigr(#[from] sea_orm_migration::DbErr),

    #[error(transparent)]
    Token(#[from] biscuit_auth::error::Token),
}

fn err_to_status(e: Error) -> tonic::Status {
    match e {
        x => tonic::Status::internal(x.to_string()),
    }
}

pub type Result<T> = miette::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    db_uri: String,
    addr: String,
    private_key: String,
}

impl Config {
    pub fn set_listen_addr(&mut self, new_addr: String) {
        self.addr = new_addr;
    }
}

#[instrument(name = "init_config")]
pub fn init_config(config_file: Option<String>) -> Result<()> {
    let path = config_file.unwrap_or(String::from("/etc/opc/tenantd.yaml"));
    let cfg = Config {
        db_uri: String::from("postgres://postgres:postgres@localhost/tenantd"),
        addr: String::from("[::1]:50000"),
        private_key: KeyPair::new().private().to_bytes_hex().to_string(),
    };
    let mut f = File::create(&path)?;
    serde_yaml::to_writer(&mut f, &cfg)?;

    Ok(())
}

#[instrument(name = "read-config")]
pub fn read_config(config_file: Option<String>) -> Result<Config> {
    use config::File;
    let mut builder = config::ConfigBuilder::<DefaultState>::default()
        .set_default("addr", "[::1]:50000")?
        .set_default("db_uri", "postgres://postgres:postgres@localhost/tenantd")?
        .add_source(File::with_name("/etc/opc/tenantd.yaml").required(false));

    if let Some(conf_file) = config_file {
        builder = builder.add_source(File::with_name(&conf_file));
    }

    let conf = builder.build()?;
    Ok(conf.try_deserialize()?)
}

#[instrument(name = "listen")]
pub async fn listen(cfg: Config) -> Result<()> {
    let addr = cfg.addr.parse()?;
    let private_key = PrivateKey::from_bytes_hex(&cfg.private_key)?;
    let server_key = KeyPair::from(&private_key);
    let connect_options = ConnectOptions::new(cfg.db_uri).to_owned();
    let db = Database::connect(connect_options).await?;
    Migrator::up(&db, None).await?;
    let tenant_svc = TenantSvc { server_key, db };

    Server::builder()
        .add_service(TenantServer::new(tenant_svc))
        .serve(addr)
        .await?;
    Ok(())
}

#[derive(Default, Debug)]
struct TenantSvc {
    server_key: KeyPair,
    db: DatabaseConnection,
}

fn extract_token<T, KP>(
    request: tonic::Request<T>,
    kp: KP,
) -> Result<(T, Option<biscuit_auth::Biscuit>)>
where
    KP: RootKeyProvider,
{
    if let Some(token) = request.metadata().get("authorization") {
        let bisc = biscuit_auth::Biscuit::from_base64(token, kp)?;
        Ok((request.into_inner(), Some(bisc)))
    } else {
        Ok((request.into_inner(), None))
    }
}

#[tonic::async_trait]
impl Tenant for TenantSvc {
    #[doc = " A small rpc to ping to make sure we are connected, but also"]
    #[doc = " to help make a fast development function"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    #[instrument(name = "ping_handler", skip(self))]
    async fn ping(
        &self,
        request: tonic::Request<PingMsg>,
    ) -> std::result::Result<tonic::Response<PongMsg>, tonic::Status> {
        let (msg, maybe_token) =
            extract_token(request, &self.server_key.public()).map_err(err_to_status)?;

        if let Some(token) = maybe_token {
            debug!(?token, "Received token");
        }

        let resp_msg = PongMsg { pong: msg.sender };
        Ok(tonic::Response::new(resp_msg))
    }

    #[doc = " Tenant Public API"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn list_tenants(
        &self,
        request: tonic::Request<ListTenantRequest>,
    ) -> std::result::Result<tonic::Response<ListTenantResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn get_tenant(
        &self,
        request: tonic::Request<GetTenantRequest>,
    ) -> std::result::Result<tonic::Response<TenantResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn create_tenant(
        &self,
        request: tonic::Request<CreateTenantRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn delete_tenant(
        &self,
        request: tonic::Request<DeleteTenantRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[doc = " Principal Public API"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn list_principals(
        &self,
        request: tonic::Request<ListPrincipalRequest>,
    ) -> std::result::Result<tonic::Response<ListPrincipalResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn get_principal(
        &self,
        request: tonic::Request<GetPrincipalRequest>,
    ) -> std::result::Result<tonic::Response<PrincipalResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn create_principal(
        &self,
        request: tonic::Request<CreatePrincipalRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn add_public_key_to_principal(
        &self,
        request: tonic::Request<AddPublicKeyRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn remove_public_key(
        &self,
        request: tonic::Request<RemovePublicKeyRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn delete_principal(
        &self,
        request: tonic::Request<DeletePrincipalRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn get_server_public_key(
        &self,
        request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<PublicKeyResponse>, tonic::Status> {
        todo!()
    }

    //missing: `add_role`, `remove_role`, `add_attribute`, `remove_attribute`
    async fn add_role(
        &self,
        request: tonic::Request<RoleRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    async fn remove_role(
        &self,
        request: tonic::Request<RoleRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    async fn add_attribute(
        &self,
        request: tonic::Request<AttributeRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    async fn remove_attribute(
        &self,
        request: tonic::Request<AttributeRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }
}
