#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate common;
extern crate dotenv;
extern crate uuid;

mod database;
mod rpc;

use crate::database::tenant::{
    create_tenant, delete_tenant, get_tenant, list_tenants, TenantInput
};
use crate::database::principal::{
    create_principal, delete_principal, get_principal, list_principals_of_tenant, NewPrincipalInput,
    add_ssh_key_to_principal, remove_ssh_key_from_principal
};
use crate::rpc::tenant::*;
use common::*;
use database::PGPool;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use pasetors::claims::{Claims};
use pasetors::keys::{AsymmetricPublicKey, AsymmetricSecretKey};
use pasetors::{public, version4::V4};
use pasetors::paserk::FormatAsPaserk;
use rpc::tenant::status_response::Status as StatusResponseEnum;
use rpc::tenant::tenant_server::{Tenant, TenantServer};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;
use openssl::pkey::{PKey};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub struct TenantSvc {
    pool: PGPool,
    public_key: AsymmetricPublicKey<V4>,
    secret_key: AsymmetricSecretKey<V4>,
}

fn err_to_status(err: impl std::fmt::Display) -> Status {
    Status::internal(format!("{}", err))
}

#[tonic::async_trait]
impl Tenant for TenantSvc {
    async fn ping(&self, request: Request<PingMsg>) -> Result<Response<PongMsg>, Status> {
        // Log the ping we have gotten so we see some traffic
        let message = format!("received ping from: {}", request.into_inner().sender);
        info!("{}", &message);

        let reply = PongMsg { pong: message };

        Ok(Response::new(reply))
    }

    async fn list_tenants(
        &self,
        request: Request<ListTenantRequest>,
    ) -> Result<Response<ListTenantResponse>, Status> {
        let msg: ListTenantRequest = request.into_inner();

        let reply = match list_tenants(&self.pool, msg.limit, msg.offset) {
            Ok(res) => ListTenantResponse {
                tenants: res
                    .iter()
                    .map(|t| TenantResponse {
                        id: t.id.to_hyphenated().to_string(),
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
        request: Request<GetTenantRequest>,
    ) -> Result<Response<TenantResponse>, Status> {
        let msg = request.into_inner();

        let filter = match msg.filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let reply = match get_tenant(&self.pool, &filter.name) {
            Ok(r) => TenantResponse {
                id: r.id.to_hyphenated().to_string(),
                name: r.name,
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn create_tenant(
        &self,
        request: Request<CreateTenantRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let msg = request.into_inner();

        let reply = match create_tenant(&self.pool, &TenantInput{name: msg.name}) {
            Ok(_) => StatusResponse {
                code: StatusResponseEnum::Ok.into(),
                message: None,
            },
            Err(err) => StatusResponse {
                code: StatusResponseEnum::Error.into(),
                message: Some(format!("{}", err)),
            },
        };

        Ok(Response::new(reply))
    }

    async fn delete_tenant(
        &self,
        request: Request<DeleteTenantRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let msg = request.into_inner();

        let tenant_id_param = match Uuid::from_str(&msg.id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let reply = match delete_tenant(&self.pool, &tenant_id_param) {
            Ok(_) => StatusResponse { 
                code: StatusResponseEnum::Ok.into(), 
                message: None,
            },
            Err(err) => StatusResponse { 
                code: StatusResponseEnum::Error.into(), 
                message: Some(format!("{}", err)) 
            },
        };

        Ok(Response::new(reply))
    }
    
    async fn get_server_public_key(
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

    async fn list_principals(
        &self,
        request: Request<ListPrincipalRequest>,
    ) -> Result<Response<ListPrincipalResponse>, Status> {
        let msg = request.into_inner();

        let filter = match msg.filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => match Uuid::from_str(&i.tenant_id) {
                Ok(i) => i,
                Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
            },
        };

        let principals =  match list_principals_of_tenant(&self.pool, &filter, msg.limit, msg.offset) {
            Ok(res) => ListPrincipalResponse {
                principals: res
                    .iter()
                    .map(|t| PrincipalResponse {
                        id: t.id.to_hyphenated().to_string(),
                        name: t.p_name.clone(),
                        email: t.email.clone(),
                        email_confirmed: t.email_confirmed,
                    })
                    .collect(),
            },
            Err(err) => {
                return Err(Status::unknown(format!("{}", err)));
            }
        };

        Ok(Response::new(principals))
    }

    async fn get_principal(
        &self,
        request: Request<GetPrincipalRequest>,
    ) -> Result<Response<PrincipalResponse>, Status> {
        let msg = request.into_inner();

        let filter = match msg.filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let tenant_id =  match Uuid::from_str(&filter.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let (name, email) = if let Some(name_or_email) = filter.mail_or_name {
            match name_or_email {
                principal_filter::MailOrName::Email(email) => (None, Some(email)),
                principal_filter::MailOrName::Name(name) => (Some(name), None),
            }
        } else {
            (None, None)
        };

        let reply = match get_principal(&self.pool, &tenant_id, name, email) {
            Ok(r) => PrincipalResponse {
                id: r.id.to_hyphenated().to_string(),
                name: r.p_name,
                email: r.email,
                email_confirmed: r.email_confirmed,
            },
            Err(err) => return Err(Status::unknown(format!("{}", err))),
        };

        Ok(Response::new(reply))
    }

    async fn create_principal(
        &self,
        request: Request<CreatePrincipalRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let msg = request.into_inner();

        let tenant_id =  match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let input = NewPrincipalInput { p_name: msg.name, email: msg.email, tenant_id};

        let mail_token = if let Some(ref email) = input.email {
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
        
            let mut claims = Claims::new().map_err(err_to_status)?;
            claims.subject(&input.p_name).map_err(err_to_status)?;
            claims.add_additional("tenant", input.tenant_id.to_hyphenated().to_string()).map_err(err_to_status)?;
            claims.add_additional("email", email.clone()).map_err(err_to_status)?;
            claims.token_identifier(&rand_string).map_err(err_to_status)?;

            let mail_token = public::sign(
                &self.secret_key,
                &self.public_key,
                &claims,
                None,
                None,
            ).map_err(err_to_status)?;
            Some(mail_token)
        } else {
            None
        };

        let reply = match create_principal(&self.pool, &input, mail_token, msg.public_keys) {
            Ok(r) => {
                if let Some(email) = r.email {
                    StatusResponse { 
                        code: StatusResponseEnum::Ok.into(), 
                        message: Some(format!("verification mail sent to {}", email)), 
                    }
                } else {
                    StatusResponse { 
                        code: StatusResponseEnum::Ok.into(), 
                        message: None, 
                    }
                }
            },
            Err(err) => StatusResponse { 
                code: StatusResponseEnum::Error.into(), 
                message: Some(format!("{}", err)), 
            },
        };

        Ok(Response::new(reply))
    }

    async fn add_public_key_to_principal(
        &self,
        request: Request<AddPublicKeyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let msg = request.into_inner();

        if msg.public_key.is_none() {
            return Err(Status::invalid_argument("no public key provided"))
        }

        let tenant_id =  match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        add_ssh_key_to_principal(&self.pool, &principal.id, msg.public_key.unwrap())
            .map_err(err_to_status)?;

        Ok(Response::new(StatusResponse { code: StatusResponseEnum::Ok.into(), message: None }))
    }

    async fn remove_public_key(
        &self,
        request: Request<RemovePublicKeyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let msg = request.into_inner();

        let tenant_id =  match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        remove_ssh_key_from_principal(&self.pool, &principal.id, &msg.fingerprint)
            .map_err(err_to_status)?;

        Ok(Response::new(StatusResponse { code: StatusResponseEnum::Ok.into(), message: None }))
    }

    async fn delete_principal(
        &self,
        request: Request<DeletePrincipalRequest>,
    ) -> Result<Response<StatusResponse>, Status>{
        let msg = request.into_inner();

        let tenant_id =  match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        delete_principal(&self.pool, &principal.id, &tenant_id)
            .map_err(err_to_status)?;

        Ok(Response::new(StatusResponse { code: StatusResponseEnum::Ok.into(), message: None }))
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
        let keys_path = env::var("KEY_DIRECTORY").unwrap_or_else(|_| String::from(concat!(
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
        .unwrap_or_else(|_| panic!("could not find private_key for JWT in {}", &priv_key_path));
    let public_key_bytes = std::fs::read(pub_key_path.clone())
        .unwrap_or_else(|_| panic!("could not find public_key for JWT in {}", &pub_key_path));

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
