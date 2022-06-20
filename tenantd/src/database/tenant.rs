use crate::database::schema::tenants;
use crate::database::schema::tenants::dsl::*;
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;
use uuid::Uuid;

pub fn get_tenant(pool: &PGPool, tenant_name: &str) -> Result<Tenant> {
    let tenant = tenants::table
        .filter(tenants::name.eq(tenant_name))
        .limit(1)
        .first::<Tenant>(&pool.get()?)?;
    Ok(tenant)
}

pub fn get_tenant_by_id(pool: &PGPool, tenant_id: &Uuid) -> Result<Tenant> {
    let tenant = tenants::table
        .filter(tenants::id.eq(tenant_id))
        .limit(1)
        .first::<Tenant>(&pool.get()?)?;
    Ok(tenant)
}

pub fn list_tenants(pool: &PGPool, limit: u64, offset: u64) -> Result<Vec<Tenant>> {
    let results: Vec<Tenant>;
    if offset != 0 && limit != 0 {
        results = tenants::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Tenant>(&pool.get()?)?;
    } else if limit != 0 {
        results = tenants::table
            .limit(limit as i64)
            .load::<Tenant>(&pool.get()?)?;
    } else {
        results = tenants::table.load::<Tenant>(&pool.get()?)?;
    }

    Ok(results)
}

pub fn create_tenant(pool: &PGPool, tenant: &TenantInput) -> Result<Tenant> {
    let results = diesel::insert_into(tenants::table)
        .values(tenant)
        .get_result(&pool.get()?)?;
    Ok(results)
}

pub fn delete_tenant(pool: &PGPool, tenant_id: &Uuid) -> Result<()> {
    let target = tenants.find(tenant_id);
    diesel::delete(target).execute(&pool.get()?)?;
    Ok(())
}

#[derive(Insertable, AsChangeset)]
#[table_name = "tenants"]
pub struct TenantInput {
    pub name: String,
}

#[derive(Queryable, Debug)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
}
