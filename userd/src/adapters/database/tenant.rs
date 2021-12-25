use crate::database::schema::tenants;
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;

pub struct TenantRepository {
    pool: PGPool,
}

impl TenantRepository {
    pub fn get_tenant(&self, name: &str) -> Result<Tenant> {
        let tenant = tenants::table
            .filter(tenants::name.eq(name))
            .limit(1)
            .first::<Tenant>(&self.pool.get()?)?;
        Ok(tenant)
    }

    pub fn tenants(&self, limit: usize, offset: usize) -> Result<Vec<Tenant>> {
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
            results = tenants::table.load::<Tenant>(&self.pool.get()?)?;
        }

        Ok(results)
    }

    pub fn add_tenant(&self, tenant: &NewTenant) -> Result<Tenant> {
        let results = diesel::insert_into(tenants::table)
            .values(tenant)
            .get_result(&self.pool.get()?)?;
        Ok(results)
    }

    pub fn new(pool: PGPool) -> Self {
        TenantRepository { pool }
    }
}

#[derive(Insertable)]
#[table_name = "tenants"]
pub struct NewTenant<'a> {
    pub name: &'a String,
}

pub struct Tenant {
    pub id: Uuid,
    pub name: String,
}
