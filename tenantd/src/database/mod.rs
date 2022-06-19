pub mod schema;
pub mod tenant;
pub mod principal;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::sync::Arc;

pub type PGPool = Arc<Pool<ConnectionManager<PgConnection>>>;
