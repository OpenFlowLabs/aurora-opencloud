use crate::database::schema::tenants;
use crate::database::schema::tenants::dsl::*;
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;
use uuid::Uuid;

pub fn get_tenant(pool: &PGPool, tenant_name: &str) -> Fallible<TenantResponse> {
    let tenant = tenants::table
        .filter(tenants::name.eq(tenant_name))
        .limit(1)
        .first::<TenantResponse>(&pool.get()?)?;
    Ok(tenant)
}

pub fn list_tenants(pool: &PGPool, limit: u64, offset: u64) -> Fallible<Vec<TenantResponse>> {
    let results: Vec<TenantResponse>;
    if offset != 0 && limit != 0 {
        results = tenants::table
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<TenantResponse>(&pool.get()?)?;
    } else if limit != 0 {
        results = tenants::table
            .limit(limit as i64)
            .load::<TenantResponse>(&pool.get()?)?;
    } else {
        results = tenants::table.load::<TenantResponse>(&pool.get()?)?;
    }

    Ok(results)
}

pub fn update_tenant(pool: &PGPool, uuid: &Uuid, input: &TenantInput) -> Fallible<TenantResponse> {
    let target = tenants::table.find(uuid);
    let resp = diesel::update(target)
        .set(input)
        .get_result::<TenantResponse>(&pool.get()?)?;

    Ok(resp)
}

pub fn create_tenant(pool: &PGPool, tenant: &TenantInput) -> Fallible<TenantResponse> {
    let results = diesel::insert_into(tenants::table)
        .values(tenant)
        .get_result(&pool.get()?)?;
    Ok(results)
}

pub fn delete_tenant(pool: &PGPool, tenant_id: &Uuid) -> Fallible<()> {
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
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
}
