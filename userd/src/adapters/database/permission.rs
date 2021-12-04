use uuid::Uuid;
use crate::adapters::database::common::PGPool;

pub struct PermissionRepository {
    pool: PGPool
}

impl PermissionRepository {
    pub fn new(pool: PGPool) -> Self {
        PermissionRepository {
            pool
        }
    }
}

pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<String>
}