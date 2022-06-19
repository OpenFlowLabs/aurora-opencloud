pub mod principal;
pub mod schema;
pub mod tenant;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::sync::Arc;

pub type PGPool = Arc<Pool<ConnectionManager<PgConnection>>>;
