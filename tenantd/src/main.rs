#[macro_use]
extern crate diesel;
extern crate failure;

extern crate chrono;
extern crate common;
extern crate dotenv;
extern crate uuid;

/*mod rpc {
    include!(concat!("../protos.out", concat!("/", "tenant", ".rs"))); // The string specified here must match the proto package name
}*/

mod database;
mod rpc;

use crate::rpc::tenant::get_request::Filter;
use crate::rpc::tenant::{TenantFilter, TenantOperationRequestSchema};
use common::*;
use database::PGPool;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use josekit::jws::EdDSA;
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
use crate::database::tenant::{create_tenant, delete_tenant, get_tenant, list_tenants, TenantInput, TenantResponse, update_tenant};
use crate::rpc::tenant::operation_request::ObjectSchema;
use crate::rpc::tenant::operation_response::Object;


#[derive(Fail)]
enum TenantServiceError {
    #[fail(display = "invalid input type passed to function")]
    InvalidInput
}

pub struct TenantSvc {
    pool: PGPool,
}

fn tenant_response_schema_from_tenant_respose(r: TenantResponse) -> TenantResponseSchema {
    TenantResponseSchema{ uuid: r.id.to_string(), name: r.name }
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

        let result = match list_tenants(&self.pool, msg.limit, msg.offset)
        {
            Ok(res) => res,
            Err(err) => {
                return Err(Status::internal(format!("{}", err)));
            }
        };

        let reply = ListTenantResponse {
            tenant: result
                .iter()
                .map(|t| TenantResponseSchema {
                    uuid: t.id.to_string(),
                    name: t.name.clone(),
                })
                .collect(),
        };

        Ok(Response::new(reply))
    }

    async fn get_tenant(
        &self,
        request: Request<GetRequest>,
    ) -> Result<Response<TenantResponseSchema>, Status> {
        let msg = request.into_inner();

        let filter = match msg.filter {
            None => None,
            Some(f) => match f {
                Filter::User(_) => None,
                Filter::Tenant(ft) => Some(ft),
            },
        };

        let result = match get_tenant(
            &self.pool,
            &filter
                .unwrap_or(TenantFilter {
                    name: "".to_string(),
                })
                .name,
        ) {
            Ok(r) => r,
            Err(err) => return Err(Status::internal(format!("{}", err))),
        };

        let reply = TenantResponseSchema {
            uuid: result.id.to_string(),
            name: result.name,
        };

        Ok(Response::new(reply))
    }

    async fn create_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let tenant_input = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(ti) => TenantInput{ name: ti.name.clone().string() },
                ObjectSchema::User(_) => None
            }
        };

        let result = match create_tenant(&self.pool, &TenantInput{ name: tenant_input.ok_or(TenantServiceError::InvalidInput)?}) {
            Ok(r) => r,
            Err(err) => return Err(Status::internal(format!("{}", err))),
        };

        let reply = OperationResponse{
            status: rpc::tenant::operation_response::Status::Success.into(),
            response_message: "".to_string(),
            object: Some(Object::Tenant(tenant_response_schema_from_tenant_respose(result))),
        };

        Ok(Response::new(reply))
    }

    async fn update_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        let tenant_input: TenantInput = match msg.object_schema {
            None => None,
            Some(i) => match i {
                ObjectSchema::Tenant(ti) => TenantInput{ name: ti.name.clone().string() },
                ObjectSchema::User(_) => None
            }
        }.ok_or(TenantServiceError::InvalidInput)?;

        let result = match update_tenant(&self.pool, &Uuid::from_str(&msg.uuid)?, &tenant_input) {
            Ok(r) => r,
            Err(err) => return Err(Status::internal(format!("{}", err))),
        };

        let reply = OperationResponse{
            status: rpc::tenant::operation_response::Status::Success.into(),
            response_message: "".to_string(),
            object: Some(Object::Tenant(tenant_response_schema_from_tenant_respose(result))),
        };

        Ok(Response::new(reply))
    }

    async fn delete_tenant(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        let msg = request.into_inner();

        match delete_tenant(&self.pool, &Uuid::from_str(&msg.uuid)?) {
            Ok(_) => {},
            Err(err) => return Err(Status::internal(format!("{}", err))),
        };

        let reply = OperationResponse{
            status: rpc::tenant::operation_response::Status::Success.into(),
            response_message: "".to_string(),
            object: None,
        };

        Ok(Response::new(reply))
    }

    #[doc = " User CRUD"]
    async fn list_users(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<ListUserResponse>, Status> {
        Ok(Response::new(ListUserResponse::default()))
    }
    async fn get_user(
        &self,
        request: Request<GetRequest>,
    ) -> Result<Response<UserResponseSchema>, Status> {
        Ok(Response::new(UserResponseSchema::default()))
    }
    async fn create_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        Ok(Response::new(OperationResponse::default()))
    }
    async fn update_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        Ok(Response::new(OperationResponse::default()))
    }
    async fn delete_user(
        &self,
        request: Request<OperationRequest>,
    ) -> Result<Response<OperationResponse>, Status> {
        Ok(Response::new(OperationResponse::default()))
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
async fn main() -> Fallible<()> {
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
