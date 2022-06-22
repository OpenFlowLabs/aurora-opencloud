#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate common;
extern crate dotenv;
extern crate uuid;

mod database;
mod rpc;

use crate::database::principal::{
    add_ssh_key_to_principal, create_principal, delete_principal, get_principal,
    get_principal_with_key, list_principals_of_tenant, remove_ssh_key_from_principal,
    NewPrincipalInput,
};
use crate::database::tenant::{
    create_tenant, delete_tenant, get_tenant, list_tenants, TenantInput,
};
use crate::rpc::tenant::*;
use common::*;
use core::convert::TryFrom;
use database::PGPool;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use openssl::pkey::{PKey, Private};
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::footer::Footer;
use pasetors::keys::{AsymmetricPublicKey, AsymmetricSecretKey};
use pasetors::paserk::FormatAsPaserk;
use pasetors::token::UntrustedToken;
use pasetors::{public, version4::V4, Public};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rpc::tenant::status_response::Status as StatusResponseEnum;
use rpc::tenant::tenant_server::{Tenant, TenantServer};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

pub struct TenantSvc {
    pool: PGPool,
    public_key: AsymmetricPublicKey<V4>,
    secret_key: AsymmetricSecretKey<V4>,
}

fn err_to_status(err: impl std::fmt::Debug + std::fmt::Display) -> Status {
    Status::internal(format!("{}", dbg!(err)))
}

fn err_to_unauthenticated(err: impl std::fmt::Debug + std::fmt::Display) -> Status {
    Status::unauthenticated(format!("{}", dbg!(err)))
}

#[allow(dead_code)]
fn get_claim_uuid(claims: &Claims, claim_name: &str) -> Result<Option<Uuid>> {
    let id_value = claims.get_claim(claim_name);
    if let Some(id_value) = id_value {
        let id_string = id_value.as_str();
        if let Some(id_string) = id_string {
            let id = Uuid::from_str(id_string)?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn parse_claim_subject(claim: Option<&serde_json::Value>) -> Option<(String, String)> {
    if let Some(subject) = claim {
        if let Some(subject) = subject.as_str() {
            let split = subject.split_once('@');
            if let Some((principal, tenant)) = split {
                return Some((principal.to_owned(), tenant.to_owned()));
            }
        }
    }
    None
}

fn make_mail_token(
    email: Option<String>,
    principal_name: &str,
    tenant_id: &Uuid,
    private_key: &AsymmetricSecretKey<V4>,
    public_key: &AsymmetricPublicKey<V4>,
) -> Result<Option<String>> {
    let mail_token = if let Some(ref email) = email {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        let mut claims = Claims::new().map_err(err_to_status)?;
        claims.subject(principal_name).map_err(err_to_status)?;
        claims
            .add_additional("tenant", tenant_id.to_hyphenated().to_string())
            .map_err(err_to_status)?;
        claims
            .add_additional("email", email.clone())
            .map_err(err_to_status)?;
        claims
            .token_identifier(&rand_string)
            .map_err(err_to_status)?;

        let mail_token =
            public::sign(private_key, public_key, &claims, None, None).map_err(err_to_status)?;
        Some(mail_token)
    } else {
        None
    };

    Ok(mail_token)
}

#[tonic::async_trait]
impl Tenant for TenantSvc {
    async fn ping(&self, request: Request<PingMsg>) -> Result<Response<PongMsg>, Status> {
        // See if we have an Authenticated ping
        let (msg, claims) = if request.metadata().get("authorization").is_some() {
            let (msg, claims) = self.authenticate_and_extract_message(request)?;
            (msg, Some(claims))
        } else {
            (request.into_inner(), None)
        };

        // Log the ping we have gotten so we see some traffic
        let message = if let Some(claims) = claims {
            let (principal, tenant) = parse_claim_subject(claims.get_claim("sub"))
                .ok_or_else(|| Status::invalid_argument("bad id"))?;

            let tenant = get_tenant(&self.pool, &tenant).map_err(err_to_status)?;

            let principal = get_principal(&self.pool, &tenant.id, None, Some(principal))
                .map_err(err_to_status)?;

            let message = format!(
                "received authenticated ping from: {}, by principal {}@{}",
                msg.sender, principal.p_name, tenant.name
            );
            info!("{}", &message);
            message
        } else {
            let message = format!("received ping from: {}", msg.sender);
            info!("{}", &message);
            message
        };
        info!("{}", &message);

        let reply = PongMsg { pong: message };

        Ok(Response::new(reply))
    }

    async fn list_tenants(
        &self,
        request: Request<ListTenantRequest>,
    ) -> Result<Response<ListTenantResponse>, Status> {
        let (msg, _) = self.authenticate_and_extract_message(request)?;

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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let reply = match create_tenant(&self.pool, &TenantInput { name: msg.name }) {
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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

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
                message: Some(format!("{}", err)),
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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let filter = match msg.filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => match Uuid::from_str(&i.tenant_id) {
                Ok(i) => i,
                Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
            },
        };

        let principals = match list_principals_of_tenant(&self.pool, &filter, msg.limit, msg.offset)
        {
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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let filter = match msg.filter {
            None => return Err(Status::invalid_argument("input expected")),
            Some(i) => i,
        };

        let tenant_id = match Uuid::from_str(&filter.tenant_id) {
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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let tenant_id = match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let input = NewPrincipalInput {
            p_name: msg.name,
            email: msg.email,
            tenant_id,
        };

        let mail_token = make_mail_token(
            input.email.clone(),
            &input.p_name,
            &tenant_id,
            &self.secret_key,
            &self.public_key,
        )
        .map_err(err_to_status)?;

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
            }
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
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let tenant_id = match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        add_ssh_key_to_principal(&self.pool, &principal.id, &msg.public_key)
            .map_err(err_to_status)?;

        Ok(Response::new(StatusResponse {
            code: StatusResponseEnum::Ok.into(),
            message: None,
        }))
    }

    async fn remove_public_key(
        &self,
        request: Request<RemovePublicKeyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let tenant_id = match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        remove_ssh_key_from_principal(&self.pool, &principal.id, &msg.fingerprint)
            .map_err(err_to_status)?;

        Ok(Response::new(StatusResponse {
            code: StatusResponseEnum::Ok.into(),
            message: None,
        }))
    }

    async fn delete_principal(
        &self,
        request: Request<DeletePrincipalRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let (msg, _) = self.authenticate_and_extract_message(request)?;

        let tenant_id = match Uuid::from_str(&msg.tenant_id) {
            Ok(i) => i,
            Err(err) => return Err(Status::invalid_argument(format!("{}", err))),
        };

        let principal = get_principal(&self.pool, &tenant_id, Some(msg.principal_name), None)
            .map_err(err_to_status)?;

        delete_principal(&self.pool, &principal.id, &tenant_id).map_err(err_to_status)?;

        Ok(Response::new(StatusResponse {
            code: StatusResponseEnum::Ok.into(),
            message: None,
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

    fn authenticate_and_extract_message<T>(&self, req: Request<T>) -> Result<(T, Claims), Status> {
        let client_token = req
            .metadata()
            .get(AUTHORIZATION_HEADER)
            .ok_or_else(|| Status::unauthenticated("No auth token provided"))?;

        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Public, V4>::try_from(
            client_token.to_str().map_err(err_to_unauthenticated)?,
        )
        .map_err(err_to_unauthenticated)?;

        let mut footer = Footer::new();
        footer
            .parse_bytes(untrusted_token.untrusted_footer())
            .map_err(err_to_unauthenticated)?;

        //TODO: Authorization
        let claims = if footer.contains_claim("fingerprint") {
            let fingerprint_value = footer
                .get_claim("fingerprint")
                .ok_or_else(|| Status::unauthenticated("No claims in the token"))?;

            let fingerprint = fingerprint_value
                .as_str()
                .ok_or_else(|| Status::unauthenticated("No claims in the token"))?;

            let principal =
                get_principal_with_key(&self.pool, fingerprint).map_err(err_to_unauthenticated)?;

            let ossl_public_key =
                openssl::pkey::PKey::public_key_from_pem(principal.public_key_pem.as_bytes())
                    .map_err(err_to_unauthenticated)?;
            let raw_key_bytes = ossl_public_key
                .raw_public_key()
                .map_err(err_to_unauthenticated)?;

            let public_key =
                AsymmetricPublicKey::<V4>::from(&raw_key_bytes).map_err(err_to_unauthenticated)?;

            let trusted_token =
                public::verify(&public_key, &untrusted_token, &validation_rules, None, None)
                    .map_err(err_to_unauthenticated)?;

            trusted_token
                .payload_claims()
                .ok_or_else(|| Status::unauthenticated("No claims in the token"))?
                .clone()
        } else {
            let trusted_token = public::verify(
                &self.public_key,
                &untrusted_token,
                &validation_rules,
                None,
                None,
            )
            .map_err(err_to_unauthenticated)?;

            trusted_token
                .payload_claims()
                .ok_or_else(|| Status::unauthenticated("No claims in the token"))?
                .clone()
        };

        Ok((req.into_inner(), claims))
    }
}

// TODO: Embed migrations
// https://docs.diesel.rs/1.4.x/diesel_migrations/macro.embed_migrations.html

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: CliCommands,

    #[clap(long, short, env, value_parser)]
    database_url: Option<String>,

    #[clap(long, short, env, value_parser)]
    key_directory: Option<String>,
}

#[derive(Subcommand, Debug)]
enum CliCommands {
    // Run the server
    #[clap(about)]
    Serve {
        #[clap(long, short, default_value_t = String::from("127.0.0.1"), value_parser)]
        listen: String,

        #[clap(long, short, default_value_t = String::from("50051"), value_parser)]
        port: String,
    },
    // Create a new tenant inside the database
    #[clap(about)]
    CreateTenant {
        #[clap(value_parser)]
        name: String,
    },
    // Create a Principal inside the database
    #[clap(about)]
    CreatePrincipal {
        #[clap(value_parser)]
        tenant: String,

        #[clap(value_parser)]
        principal_name: String,

        #[clap(value_parser)]
        public_key: String,

        #[clap(value_parser)]
        email: Option<String>,
    },
}

fn load_keys_from_disk<P: AsRef<Path>>(
    key_directory: P,
) -> Result<(PKey<Private>, PKey<openssl::pkey::Public>)> {
    info!("Loading Signature keys");
    let (pub_key_path, priv_key_path) = {
        (
            key_directory.as_ref().join("ED25519_public.pem"),
            key_directory.as_ref().join("ED25519_private.pem"),
        )
    };

    let private_key_bytes = std::fs::read(priv_key_path)?;
    let public_key_bytes = std::fs::read(pub_key_path)?;

    let private_key = PKey::private_key_from_pem(&private_key_bytes)?;
    let public_key = PKey::public_key_from_pem(&public_key_bytes)?;

    Ok((private_key, public_key))
}

fn build_database_connection(database_url: &str) -> Result<PGPool> {
    info!("Initiating Database connection");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(Pool::builder().max_size(15).build(manager)?);
    Ok(pool)
}

async fn serve<P: AsRef<Path>>(
    listen: &str,
    port: &str,
    database_url: &str,
    key_directory: P,
) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", listen, port).parse()?;

    let (private_key, public_key) = load_keys_from_disk(key_directory)?;

    let pool = build_database_connection(database_url)?;

    let tenant_service = TenantSvc::new(
        pool,
        public_key.raw_public_key()?,
        private_key.raw_private_key()?,
    )?;
    info!("Starting Tenant Service");
    Server::builder()
        .add_service(TenantServer::new(tenant_service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _guard = init_slog_logging(false)?;

    dotenv().ok();

    let key_directory = if let Some(key_directory) = cli.key_directory {
        PathBuf::from(key_directory)
    } else {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
        let manifest_path = Path::new(&manifest_dir);
        manifest_path.join("sample_data").join("keys")
    };

    let database_url = if let Some(database_url) = cli.database_url {
        database_url
    } else {
        env::var("DATABASE_URL")?
    };

    match cli.command {
        CliCommands::Serve { listen, port } => {
            serve(&listen, &port, &database_url, &key_directory).await
        }
        CliCommands::CreateTenant { name } => {
            let pool = build_database_connection(&database_url)?;
            let tenant = create_tenant(&pool, &TenantInput { name })?;
            info!(
                "Created Tenant {} with id {}",
                tenant.name,
                tenant.id.to_hyphenated()
            );
            Ok(())
        }
        CliCommands::CreatePrincipal {
            tenant,
            principal_name,
            email,
            public_key,
        } => {
            let pool = build_database_connection(&database_url)?;
            let (private_key, server_public_key) = load_keys_from_disk(key_directory)?;
            let (private_key, server_public_key) = (
                AsymmetricSecretKey::<V4>::from(&private_key.raw_private_key()?)?,
                AsymmetricPublicKey::<V4>::from(&server_public_key.raw_public_key()?)?,
            );

            debug!("Getting tenant from DB");
            let tenant = get_tenant(&pool, &tenant)?;
            debug!("tenant id is {}, with name {}", &tenant.id, &tenant.name);

            let input = NewPrincipalInput {
                p_name: principal_name.clone(),
                email: email.clone(),
                tenant_id: tenant.id,
            };

            debug!("Generating email token");
            let mail_token = make_mail_token(
                email,
                &principal_name,
                &tenant.id,
                &private_key,
                &server_public_key,
            )?;

            info!("Loading Public key");
            let public_key = if Path::exists(Path::new(&public_key)) {
                debug!("Reading keyfile {}", &public_key);
                fs::read_to_string(Path::new(&public_key))?
            } else {
                public_key
            };

            info!("Adding Principal to DB");
            let principal = create_principal(&pool, &input, mail_token, vec![public_key])?;

            info!("Created Principal {}@{}", principal.p_name, tenant.name);
            Ok(())
        }
    }
}
