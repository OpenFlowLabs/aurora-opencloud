use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;

pub type PGPool = Arc<Pool<ConnectionManager<PgConnection>>>;