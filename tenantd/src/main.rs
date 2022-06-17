#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate common;
extern crate dotenv;
extern crate uuid;

mod database;
mod rpc;

use crate::database::tenant::{
    create_tenant, delete_tenant, get_tenant, list_tenants, update_tenant, TenantInput,
    TenantResponse,
};
use crate::database::user::{
    create_user, delete_user, get_user, list_users, update_user, DBUser, NewUser,
};
use crate::rpc::tenant::get_request::Filter;
use crate::rpc::tenant::operation_request::ObjectSchema;
use crate::rpc::tenant::operation_response::Object;
use crate::rpc::tenant::{list_request, LoginRequest, LoginResponse, PublicKeyResponse};
use common::*;
use database::PGPool;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use pasetors::claims::{Claims};
use pasetors::keys::{AsymmetricPublicKey, AsymmetricSecretKey};
use pasetors::{public, version4::V4};
use pasetors::paserk::FormatAsPaserk;
use pwhash::bcrypt;
use rpc::tenant::tenant_server::{Tenant, TenantServer};
use rpc::tenant::{
    GetRequest, ListRequest, ListTenantResponse, ListUserResponse, OperationRequest,
    OperationResponse, PingMsg, PongMsg, TenantResponseSchema, UserResponseSchema,
};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use openssl::pkey::{PKey};


pub struct TenantSvc {
    pool: PGPool,
    public_key: AsymmetricPublicKey<V4>,
    secret_key: AsymmetricSecretKey<V4>,
}

fn tenant_response_schema_from_tenant_respose(r: TenantResponse) -> TenantResponseSchema {
    TenantResponseSchema {
        uuid: r.id.to_string(),
        name: r.name,
    }
}

fn empty_to_optional_string(input: &str) -> Option<String> {
    if input == "" {
        None
    } else {
        Some(input.to_string())
    }
}

#[tonic::async_trait]
impl Tenant for TenantSvc {
    async fn ping(&self, request: Request<PingMsg>) -> Result<Response<PongMsg>, Status> {
        // Log the ping we have gotten so we see some traffic
        info!("received ping from: {}", request.into_inner().sender);

        let reply = PongMsg {
            status: rpc::tenant::pong_msg::Status::Success.into(),
        };

        Ok(Response::new(reply))
    }

    async fn list_tenants(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<ListTenantResponse>, Status> {
        let msg: ListRequest = request.into_inner();

        let reply = match list_tenants(&self.pool, msg.limit, msg.offset) {
            Ok(res) => ListTenantResponse {
                tenants: res
                    .iter()
                    .map(|t| TenantResponseSchema {
                        uuid: t.id.to_string(),
                        name: t.name.clone(),
                    })
                    .collect(),
            },
            Err(err) => {
                return Err(Status::unknown(format!("{}", err)));
            }
        };

        Ok(Response::new(reply))
    }

    async fn get_tenant(
        &self,
        request: Request<GetRequest>,
    ) -> Result<Response<TenantResponseSchema>, Status> {
        let msg = request.into_inner();

        let maybe_filter = match msg.filter {
            None => None,
            Some(f) => match f {
                Filter::User(_) => None,
                Filter::Tenant(ft) => Some(ft),
            },
        };

        let filter = match maybe_filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match get_tenant(&self.pool, &filter.name) {
            Ok(r) => TenantResponseSchema {
                uuid: r.id.to_string(),
                name: r.name,
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn create_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let maybe_tenant_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(ti) => Some(TenantInput {
                    name: ti.name.clone(),
                }),
                ObjectSchema::User(_) => None,
            },
        };

        let tenant_input = match maybe_tenant_input {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match create_tenant(&self.pool, &tenant_input) {
            Ok(r) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: Some(Object::Tenant(tenant_response_schema_from_tenant_respose(
                    r,
                ))),
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn update_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let maybe_tenant_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(ti) => Some(TenantInput {
                    name: ti.name.clone(),
                }),
                ObjectSchema::User(_) => None,
            },
        };

        let tenant_input = match maybe_tenant_input {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let tenant_id_param = match Uuid::from_str(&msg.uuid) {
            Ok(i) => i,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        let reply = match update_tenant(&self.pool, &tenant_id_param, &tenant_input) {
            Ok(r) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: Some(Object::Tenant(tenant_response_schema_from_tenant_respose(
                    r,
                ))),
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn delete_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let tenant_id_param = match Uuid::from_str(&msg.uuid) {
            Ok(i) => i,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        let reply = match delete_tenant(&self.pool, &tenant_id_param) {
            Ok(_) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: None,
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    #[doc = " User CRUD"]
    async fn list_users(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<ListUserResponse>, Status> {
        let msg: ListRequest = request.into_inner();

        let maybe_filter = match msg.filter {
            None => None,
            Some(f) => match f {
                list_request::Filter::User(fu) => Some(fu),
                list_request::Filter::Tenant(_) => None,
            },
        };

        let filter = match maybe_filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let tenant_id_param = match Uuid::from_str(&filter.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        let reply = match list_users(&self.pool, &tenant_id_param, msg.limit, msg.offset) {
            Ok(res) => ListUserResponse {
                users: res
                    .iter()
                    .map(|t| Self::dbuser_to_user_response_schema(t))
                    .collect(),
            },
            Err(err) => {
                return Err(Status::unknown(format!("{}", err)));
            }
        };

        Ok(Response::new(reply))
    }
    async fn get_user(
        &self,
        request: Request<GetRequest>,
    ) -> Result<Response<UserResponseSchema>, Status> {
        let msg: GetRequest = request.into_inner();

        let maybe_filter = match msg.filter {
            None => None,
            Some(f) => match f {
                Filter::User(fu) => Some(fu),
                Filter::Tenant(_) => None,
            },
        };

        let filter = match maybe_filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let tenant_id_param = match Uuid::from_str(&filter.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        let reply = match get_user(
            &self.pool,
            &tenant_id_param,
            empty_to_optional_string(&filter.email),
            empty_to_optional_string(&filter.username),
        ) {
            Ok(res) => Self::dbuser_to_user_response_schema(&res),
            Err(err) => {
                return Err(Status::unknown(format!("{}", err)));
            }
        };

        Ok(Response::new(reply))
    }
    async fn create_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let maybe_user_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(_) => None,
                ObjectSchema::User(ui) => match Uuid::from_str(&ui.tenant_id).ok() {
                    None => None,
                    Some(tenant_id_param) => Some(NewUser {
                        username: ui.username.clone(),
                        email: ui.email.clone(),
                        tenant_id: tenant_id_param,
                        pwhash: match bcrypt::hash(ui.password) {
                            Ok(v) => v,
                            Err(err) => return Err(Status::internal(format!("{}", err))),
                        },
                    }),
                },
            },
        };

        let user_input = match maybe_user_input {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match create_user(&self.pool, &user_input) {
            Ok(r) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: Some(Object::User(UserResponseSchema {
                    tenant_id: r.0.tenant_id.to_string(),
                    uuid: r.0.id.to_string(),
                    username: r.0.username.clone(),
                    email: r.0.email.clone(),
                    email_confirmed: r.0.email_confirmed.clone(),
                })),
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }
    async fn update_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let maybe_user_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(_) => None,
                ObjectSchema::User(ui) => match Uuid::from_str(&ui.tenant_id).ok() {
                    None => None,
                    Some(tenant_id_param) => match Uuid::from_str(&msg.uuid).ok() {
                        None => None,
                        Some(user_uuid) => Some((
                            tenant_id_param,
                            user_uuid,
                            empty_to_optional_string(&ui.username),
                            empty_to_optional_string(&ui.email),
                        )),
                    },
                },
            },
        };

        let user_input = match maybe_user_input {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match update_user(
            &self.pool,
            &user_input.1,
            &user_input.0,
            user_input.2,
            user_input.3,
        ) {
            Ok(r) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: Some(Object::User(UserResponseSchema {
                    tenant_id: r.tenant_id.to_string(),
                    uuid: r.id.to_string(),
                    username: r.username.clone(),
                    email: r.email.clone(),
                    email_confirmed: r.email_confirmed.clone(),
                })),
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }
    async fn delete_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let maybe_user_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(_) => None,
                ObjectSchema::User(ui) => match Uuid::from_str(&ui.tenant_id).ok() {
                    None => None,
                    Some(tenant_id_param) => match Uuid::from_str(&msg.uuid).ok() {
                        None => None,
                        Some(user_uuid) => Some((tenant_id_param, user_uuid)),
                    },
                },
            },
        };

        let user_input = match maybe_user_input {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match delete_user(&self.pool, &user_input.1, &user_input.0) {
            Ok(_) => OperationResponse {
                status: rpc::tenant::operation_response::Status::Success.into(),
                response_message: "".to_string(),
                object: None,
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let msg: LoginRequest = request.into_inner();

        let tenant_id_param = match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        //Find User
        let db_user = match get_user(&self.pool, &tenant_id_param, Some(msg.username), None) {
            Ok(u) => u,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        let reply = match self.login_db_user(db_user, &msg.password, false) {
            Ok(token_repsonse) => token_repsonse,
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn get_public_key(
        &self,
        _request: Request<()>,
    ) -> Result<Response<PublicKeyResponse>, Status> {
        let mut paserk = String::new();

        match self.public_key.fmt(&mut paserk) {
            Ok(_) => {}
            Err(err) => return Err(Status::internal(format!("{}", err))),
        }

        Ok(Response::new(PublicKeyResponse {
            public_key: vec![paserk],
        }))
    }
}

impl TenantSvc {
    pub fn new(pool: PGPool, public_key: Vec<u8>, secret_key: Vec<u8>) -> Result<Self> {
        Ok(TenantSvc {
            pool,
            public_key: AsymmetricPublicKey::<V4>::from(&public_key)?,
            secret_key: AsymmetricSecretKey::<V4>::from(&secret_key)?,
        })
    }

    fn login_db_user(
        &self,
        user: DBUser,
        password: &str,
        remember: bool,
    ) -> Result<LoginResponse> {
        user.verify_pw_result(password)?;

        // Setup the default claims, which include `iat` and `nbf` as the current time and `exp` of one hour.
        // Add a custom `data` claim as well.
        let mut claims = Claims::new()?;
        claims.add_additional("data", "A public, signed message")?;

        let auth_token = public::sign(
            &self.secret_key,
            &self.public_key,
            &claims,
            None,
            Some(b"implicit assertion"),
        )?;

        let refresh_token = {
            if remember {
                Some(public::sign(
                    &self.secret_key,
                    &self.public_key,
                    &claims,
                    None,
                    Some(b"implicit assertion"),
                )?)
            } else {
                None
            }
        };

        Ok(LoginResponse {
            auth_token,
            refresh_token,
        })
    }

    fn dbuser_to_user_response_schema(res: &DBUser) -> UserResponseSchema {
        UserResponseSchema {
            tenant_id: res.tenant_id.to_string(),
            uuid: res.id.to_string(),
            username: res.username.clone(),
            email: res.email.clone(),
            email_confirmed: res.email_confirmed.clone(),
        }
    }
}

// TODO: Embed migrations
// https://docs.diesel.rs/1.4.x/diesel_migrations/macro.embed_migrations.html

// TODO: use clap for the daemon and initialize subcommands

// TODO: Move keys to paserk format and as strings

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:50051".parse()?;
    let logger = init_log();
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = set_global_logger(logger);

    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Loading Signature keys");
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

    //let issuer_base = env::var("ISSUER_BASE").expect("ISSUER_BASE must be set");
    let private_key_bytes = std::fs::read(priv_key_path.clone())
        .expect(format!("could not find private_key for JWT in {}", &priv_key_path).as_str());
    let public_key_bytes = std::fs::read(pub_key_path.clone())
        .expect(format!("could not find public_key for JWT in {}", &pub_key_path).as_str());

    let private_key = PKey::private_key_from_pem(&private_key_bytes)?;
    let public_key = PKey::public_key_from_pem(&public_key_bytes)?;

    info!("Initiating Database connection");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(Pool::builder().max_size(15).build(manager).unwrap());

    let tenant_service = TenantSvc::new(pool, public_key.raw_public_key()?, private_key.raw_private_key()?)?;
    info!("Starting Tenant Service");
    Server::builder()
        .add_service(TenantServer::new(tenant_service))
        .serve(addr)
        .await?;

    Ok(())
}
