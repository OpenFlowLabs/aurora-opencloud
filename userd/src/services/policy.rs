use juniper::{GraphQLInputObject, FieldResult};
use uuid::Uuid;
use crate::adapters::database::schema::policy_permissions as PolicyPermsDB;
use crate::adapters::database::schema::permissions as PermsDB;
use crate::adapters::database::schema::policies as PolicyDB;
use crate::adapters::database::common::PGPool;

use crate::diesel::prelude::*;

const RESOURCE_SCOPE_ALL: &str = "*";

pub struct PolicyService {
    pub pool: PGPool
}

impl PolicyService {
    pub fn find_policies_by_name(&self, tenant_id: &Uuid, name_search: &str) -> FieldResult<Vec<Policy>> {
        let policies = PolicyDB::dsl::policies.filter(PolicyDB::dsl::tenant_id.eq(tenant_id).and(
            PolicyDB::dsl::name.like(format!("%{}%", name_search))
        )).load::<Policy>(&self.pool.get()?)?;
        Ok(policies)
    }

    pub fn get_policy_by_id(&self, id: Uuid) -> FieldResult<Policy> {

    }

    pub fn list_policies_of_tenant(&self, tenant_id: &Uuid) -> FieldResult<Vec<Policy>> {

    }

    pub fn delete_policy(&self, policy_id:Uuid) -> FieldResult<Policy> {

    }

    pub fn get_permissions(&self, tenant_id: &Uuid, policy_id: &Uuid) -> FieldResult<Vec<String>> {
        let permissions = PolicyPermsDB::table.select(PolicyPermsDB::columns::permission).
            filter(PolicyPermsDB::dsl::tenant_id.eq(tenant_id).and(
                PolicyPermsDB::dsl::policy_id.eq(policy_id)
            )).load::<String>(&self.pool.get()?)?;
        Ok(permissions)
    }

    pub fn create_policy(&self, tenant_id: &Uuid, policy_name: &str, permissions: &Vec<String>) -> FieldResult<(Policy, Vec<String>)> {

    }

    pub fn update_policy(&self, tenant_id: &Uuid, policy_id: &Uuid, permissions: &Vec<String>) -> FieldResult<(Policy, Vec<String>)> {

    }

    pub fn apply_policy(&self, tenant_id: &Uuid, policy_id: &Uuid, user_id: &Uuid) -> FieldResult<(Policy, Vec<String>)> {

    }

    pub fn new(pool: PGPool) -> Self {
        PolicyService{
            pool
        }
    }
}

#[derive(Debug, Default, GraphQLInputObject)]
pub struct PolicyInput {
    pub name: String,
    pub permission: Vec<PermissionInput>,
}

#[derive(Debug, Default, GraphQLInputObject)]
pub struct PermissionInput {
    pub service: String,
    pub permission: String,
    pub resource_scope: Option<String>,
}

pub struct Policy {
    pub id: Uuid,
    pub name: String
}

pub fn is_string_a_valid_policy(s: &str) -> bool {
    s.matches(":").count() == 2
}

impl PermissionInput {
    pub fn new(service: &str, permission: &str, resource_scope: Option<&str>) -> Self {
        PermissionInput {
            service: service.into(),
            permission: permission.into(),
            resource_scope: if let Some(scope) = resource_scope { Some(scope.into()) } else { Some(RESOURCE_SCOPE_ALL.into()) }
        }
    }
}

impl From<String> for PermissionInput {
    fn from(s: String) -> Self {
        if !is_string_a_valid_policy(&s) {
            panic!("Invalid policy string passed to from string for PermissionPolicy: got {}", s)
        }

        let mut iter = s.splitn(3, ':');
        let service = iter.next().unwrap();
        let permission = iter.next().unwrap();
        let resource_scope = iter.next().unwrap();
        PermissionInput {
            service: service.into(),
            permission: permission.into(),
            resource_scope: Some(resource_scope.into()),
        }
    }
}

impl Into<String> for PermissionInput {
    fn into(self) -> String {
        format!("{}:{}:{}", self.service, self.permission, if let Some(scope) = &self.resource_scope {scope} else {RESOURCE_SCOPE_ALL})
    }
}