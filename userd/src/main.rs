#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate uuid;
extern crate log;
extern crate fern;

extern crate chrono;

mod services;
mod adapters;
use crate::adapters::graphql::{RootContext, Schema, QueryRoot, MutationRoot};
use rocket::{response::content, State};
use juniper::{
    EmptySubscription
};
use dotenv::dotenv;
use std::env;
use josekit::{jws::EdDSA};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use crate::services::tenant::TenantService;
use crate::services::user::UserService;
use std::sync::Arc;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[rocket::get("/")]
pub fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/graphql?<request>")]
pub fn get_graphql_handler(
    context: State<RootContext>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute_sync(&schema, &context)
}

#[rocket::post("/graphql", data = "<request>")]
pub fn post_graphql_handler(
    context: State<RootContext>,
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute_sync(&schema, &context)
}

// TODO: Embed migrations
// https://docs.diesel.rs/1.4.x/diesel_migrations/macro.embed_migrations.html

// TODO: use clap for the daemon and initialize subcommands

fn main() {
    dotenv().ok();

    setup_logger().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let (pub_key_path, priv_key_path) = {
        let keys_path = env::var("KEY_DIRECTORY")
            .unwrap_or(String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/sample_data/keys")));
        (
            keys_path.clone() + "/ED25519_public.pem",
            keys_path + "/ED25519_private.pem"
        )
    };

    let issuer_base = env::var("ISSUER_BASE")
        .expect("ISSUER_BASE must be set");

    let private_key = std::fs::read(priv_key_path.clone()).expect(format!("could not find private_key for JWT in {}", &priv_key_path).as_str());
    let public_key = std::fs::read(pub_key_path.clone()).expect(format!("could not find public_key for JWT in {}", &pub_key_path).as_str());
    let signer = EdDSA.signer_from_pem(&private_key).expect(format!("cannot make signer from private_key is it PKCS#8 formatted?").as_str());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Arc::new(Pool::builder().max_size(15).build(manager).unwrap());

    let tenant_service = TenantService::new(pool.clone());
    let user_service = UserService::new(pool.clone(), issuer_base, signer);

    let query_root = QueryRoot::new(tenant_service.clone(), user_service.clone(), public_key.clone());
    let mutation_root = MutationRoot::new(tenant_service, user_service, public_key);

    rocket::ignite()
        .manage(RootContext{})
        .manage(Schema::new(
            query_root,
            mutation_root,
            EmptySubscription::<RootContext>::new(),
        ))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch();
}