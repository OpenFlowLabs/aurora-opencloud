use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::services::tenant::Tenant;
use crate::adapters::database::schema::tenants;
use juniper::FieldResult;
use crate::adapters::database::common::PGPool;

pub struct TenantRepository {
    pool: PGPool
}

impl TenantRepository {
    pub fn get_tenant(&self, name: &str) -> FieldResult<Tenant> {
        let tenant = tenants::table
            .filter(tenants::name.eq(name))
            .limit(1)
            .first::<Tenant>(&self.pool.get()?)?;
        Ok(tenant)
    }

    pub fn tenants(&self, limit: usize, offset: usize) -> FieldResult<Vec<Tenant>> {
        let results: Vec<Tenant>;
        if offset != 0 && limit != 0 {
            results = tenants::table
                .limit(limit as i64)
                .offset(offset as i64)
                .load::<Tenant>(&self.pool.get()?)?;
        } else if limit != 0 {
            results = tenants::table
                .limit(limit as i64)
                .load::<Tenant>(&self.pool.get()?)?;
        } else {
            results = tenants::table
                .load::<Tenant>(&self.pool.get()?)?;
        }

        Ok(results)
    }

    pub fn add_tenant(&self, tenant: &NewTenant) -> FieldResult<Tenant> {
        let results = diesel::insert_into(tenants::table)
            .values(tenant)
            .get_result(&self.pool.get()?)?;
        Ok(results)
    }

    pub fn new(pool: PGPool) -> Self {
        TenantRepository{
            pool,
        }
    }
}

#[derive(Insertable)]
#[table_name="tenants"]
pub struct NewTenant<'a> {
    pub name: &'a String,
}
