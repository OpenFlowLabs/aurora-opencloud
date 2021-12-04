use uuid::Uuid;
use juniper::{
    GraphQLObject, GraphQLInputObject, FieldResult
};
use crate::adapters::database::tenant::{NewTenant, TenantRepository};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::adapters::database::common::PGPool;

#[derive(Clone)]
pub struct TenantService {
    tenant_repository: Box<TenantRepository>
}

impl TenantService {
    pub fn new(pool: PGPool) -> Self {
        TenantService  {
            tenant_repository: Box::new(TenantRepository::new(pool)),
        }
    }

    pub fn get_tenant(&self, name: &str) -> FieldResult<Option<Tenant>> {
        let result = self.tenant_repository.get_tenant(name)?;
        Ok(Some(result))
    }

    pub fn tenants(&self, limit: usize, offset: usize) -> FieldResult<Option<Vec<Tenant>>> {
        let tenants = self.tenant_repository.tenants(limit, offset)?;
        Ok(Some(tenants))
    }

    pub fn add_tenant(&self, input: TenantInput) -> FieldResult<Tenant> {
        let result = self.tenant_repository.add_tenant(&NewTenant{
            name: &input.name
        })?;
        Ok(result)
    }

    pub fn update_tenant(&self, input: TenantInput) -> FieldResult<Tenant> {

    }

    pub fn delete_tenant(&self, tenant_id: Uuid) -> FieldResult<Tenant> {

    }
}

#[derive(GraphQLInputObject)]
pub struct TenantInput {
    name: String,
    default_policy: Option<Uuid>
}

#[derive(GraphQLObject, Default, Clone, Queryable)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
}