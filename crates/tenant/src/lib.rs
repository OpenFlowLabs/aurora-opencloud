mod database;
mod migrations;
mod rpc;

use biscuit_auth::{
    macros::{authorizer, biscuit},
    Biscuit, KeyPair, PrivateKey, RootKeyProvider,
};
use config::builder::DefaultState;
use derivative::Derivative;
use miette::Diagnostic;
use migrations::{Migrator, MigratorTrait};
use rpc::tenant::{
    pong_msg::Authenticated,
    status_response::Status,
    tenant_server::{Tenant, TenantServer},
    AddPublicKeyRequest, AttributeRequest, CreatePrincipalRequest, CreateTenantRequest,
    DefineRoleRequest, DeletePrincipalRequest, DeleteTenantRequest, GetPrincipalAuthRequest,
    GetPrincipalAuthResponse, GetPrincipalRequest, GetTenantRequest, ListPrincipalRequest,
    ListPrincipalResponse, ListTenantRequest, ListTenantResponse, PingMsg, PongMsg,
    PrincipalResponse, PublicKeyResponse, RefreshTokenResponse, RemovePublicKeyRequest,
    RevokeTokenRequest, RoleRequest, StatusResponse, TenantResponse,
};
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File};
use thiserror::Error;
use tonic::transport::Server;
use tracing::{debug, instrument};
use uuid::Uuid;

enum Permissions {
    TenantRead,
    TenantCreate,
    TenantDelete,
    PrincipalCreate,
    PrincipalRead,
    PrincipalPublicKeyEdit,
    PrincipalImpersonate,
    PrincipalInvite,
    PrincipalLogout,
    PrincipalDelete,
    RoleDefine,
    RoleLink,
    PermissionDefine,
}

impl Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permissions::TenantRead => write!(f, "tenant:read"),
            Permissions::TenantCreate => write!(f, "tenant:create"),
            Permissions::TenantDelete => write!(f, "tenant:delete"),
            Permissions::PrincipalCreate => write!(f, "principal:create"),
            Permissions::PrincipalRead => write!(f, "principal:read"),
            Permissions::PrincipalPublicKeyEdit => write!(f, "principal:public_key:edit"),
            Permissions::PrincipalImpersonate => write!(f, "principal:impersonate"),
            Permissions::PrincipalInvite => write!(f, "principal:invite"),
            Permissions::PrincipalLogout => write!(f, "principal:logout"),
            Permissions::PrincipalDelete => write!(f, "principal:delete"),
            Permissions::RoleDefine => write!(f, "role:define"),
            Permissions::RoleLink => write!(f, "role:link"),
            Permissions::PermissionDefine => write!(f, "permission:define"),
        }
    }
}

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

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error("unauthenticated")]
    Unauthenticated,
}

fn err_to_status(e: Error) -> tonic::Status {
    match e {
        Error::Unauthenticated => {
            tonic::Status::unauthenticated("token authentication is mandatory")
        }
        x => tonic::Status::internal(x.to_string()),
    }
}

pub type Result<T> = miette::Result<T, Error>;

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct Config {
    db_uri: String,
    addr: String,
    #[derivative(Debug = "ignore")]
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

#[instrument(name = "read_config")]
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

#[instrument(name = "generate_init_token")]
pub fn generate_init_token(cfg: &Config) -> Result<Biscuit> {
    let pk = PrivateKey::from_bytes_hex(&cfg.private_key)?;
    let root = KeyPair::from(&pk);
    let root_tenant = uuid::Uuid::nil();
    let authority = biscuit!(
        r#"
        user(0);
        right(0, {root_tenant}, {operation});
        "#,
        operation = Permissions::TenantCreate.to_string(),
    );

    Ok(authority.build(&root)?.seal()?)
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

fn rights_resolver() -> biscuit_auth::Authorizer {
    authorizer!(
        r#"
right($id, $resource, $operation) <-
user($id),
operation($operation),
principal_roles($id, $resource, $roles),
role($role, $permissions),
$roles.contains($role),
$permissions.contains($operation);
allow if
operation($op),
right($id, $resource, $op);
        "#
    )
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

fn try_extract_token<T, KP>(
    request: tonic::Request<T>,
    kp: KP,
) -> Result<(T, biscuit_auth::Biscuit)>
where
    KP: RootKeyProvider,
{
    if let Some(token) = request.metadata().get("authorization") {
        let bisc = biscuit_auth::Biscuit::from_base64(token, kp)?;
        Ok((request.into_inner(), bisc))
    } else {
        Err(Error::Unauthenticated)
    }
}

fn authorize(token: &Biscuit) -> Result<()> {
    //TODO get roles from database
    let mut authorizer = rights_resolver();
    authorizer.add_token(token)?;
    authorizer.authorize()?;
    Ok(())
}

async fn handle_create_tenant(db: &DatabaseConnection, request: CreateTenantRequest) -> Result<()> {
    use database::tenant::ActiveModel;
    let tenant_model = ActiveModel {
        id: ActiveValue::Set(Uuid::parse_str(&request.id)?),
        name: ActiveValue::Set(request.name),
        parent: ActiveValue::Set(
            request
                .parent
                .map_or(None, |uuid_str| Uuid::parse_str(&uuid_str).ok()),
        ),
    };
    tenant_model.insert(db).await?;
    Ok(())
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
        match extract_token(request, &self.server_key.public()).map_err(err_to_status) {
            Ok((msg, maybe_token)) => {
                if let Some(token) = maybe_token {
                    debug!(?token, "Received token");

                    let resp_msg = PongMsg {
                        auth_status: Authenticated::Sucessfull.into(),
                        message: Some(msg.sender),
                    };
                    Ok(tonic::Response::new(resp_msg))
                } else {
                    let resp_msg = PongMsg {
                        auth_status: Authenticated::None.into(),
                        message: Some(msg.sender),
                    };
                    Ok(tonic::Response::new(resp_msg))
                }
            }
            Err(e) => {
                let resp_msg = PongMsg {
                    auth_status: Authenticated::Failed.into(),
                    message: Some(e.to_string()),
                };
                Ok(tonic::Response::new(resp_msg))
            }
        }
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
        let (msg, token) =
            try_extract_token(request, &self.server_key.public()).map_err(err_to_status)?;

        authorize(&token).map_err(err_to_status)?;
        handle_create_tenant(&self.db, msg)
            .await
            .map_err(err_to_status)?;
        Ok(tonic::Response::new(StatusResponse {
            code: Status::Ok.into(),
            message: None,
        }))
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
    ) -> std::result::Result<tonic::Response<PrincipalResponse>, tonic::Status> {
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

    async fn define_role(
        &self,
        request: tonic::Request<DefineRoleRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }

    async fn get_principal_auth(
        &self,
        request: tonic::Request<GetPrincipalAuthRequest>,
    ) -> std::result::Result<tonic::Response<GetPrincipalAuthResponse>, tonic::Status> {
        todo!()
    }

    async fn refresh_token(
        &self,
        request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<RefreshTokenResponse>, tonic::Status> {
        todo!()
    }

    async fn revoke_token(
        &self,
        request: tonic::Request<RevokeTokenRequest>,
    ) -> std::result::Result<tonic::Response<StatusResponse>, tonic::Status> {
        todo!()
    }
}
