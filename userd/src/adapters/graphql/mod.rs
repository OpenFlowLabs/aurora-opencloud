use crate::services::tenant::{Tenant, TenantService};
use juniper::{
    EmptySubscription, RootNode, Context,
    graphql_object, FieldResult
};
use crate::services::tenant::TenantInput;
use crate::services::user::{UserService, User, UserInput, LoginInput, TokenResponse, GrantPermissionInput, RevokePermissionInput};
use uuid::Uuid;
use crate::adapters::database::policy::{Policy as DBPolicy};
use crate::services::policy::{PolicyInput, PermissionInput};
use crate::adapters::database::schema::user_policies::columns::policy_id;
use crate::adapters::database::user::DBUser;

pub struct QueryRoot {
    tenant_service: Box<TenantService>,
    user_service: Box<UserService>,
    public_key: Vec<u8>,
}
pub struct MutationRoot {
    tenant_service: Box<TenantService>,
    user_service: Box<UserService>,
    public_key: Vec<u8>,
}

impl QueryRoot {
    pub fn new(tenant_service: TenantService, user_service: UserService, public_key: Vec<u8>) -> Self {
        QueryRoot {
         tenant_service: Box::new(tenant_service),
         user_service: Box::new(user_service),
         public_key,
        }
    }
}

impl MutationRoot {
    pub fn new(tenant_service: TenantService, user_service: UserService, public_key: Vec<u8>) -> Self {
        MutationRoot {
            tenant_service: Box::new(tenant_service),
            user_service: Box::new(user_service),
            public_key,
        }
    }
}

#[graphql_object(Context = RootContext)]
impl QueryRoot {
    #[graphql(arguments(name(description = "name of the tenant")))]
    fn tenant(&self, ctx: &RootContext, name: String) -> FieldResult<Option<Tenant>> {
        self.tenant_service.get_tenant(&name)
    }

    fn tenants(&self, ctx: &RootContext, limit: Option<i32>, offset: Option<i32>) -> FieldResult<Option<Vec<Tenant>>> {
        self.tenant_service.tenants(limit.unwrap_or(0) as usize, offset.unwrap_or(0) as usize)
    }

    fn user(&self, ctx: &RootContext, tenant: Uuid, email: Option<String>, username: Option<String>) -> FieldResult<Option<User>> {
        self.user_service.get_user(tenant, email, username)
    }

    fn users(&self, ctx: &RootContext, tenant: Uuid, limit: Option<i32>, offset: Option<i32>) -> FieldResult<Option<Vec<User>>> {
        self.user_service.users(tenant, limit.unwrap_or(0) as usize, offset.unwrap_or(0) as usize)
    }

    fn policies(&self, ctx: RootContext, name_search: Option<String>, id_list: Option<Vec<Uuid>>) -> FieldResult<Vec<DBPolicy>> {

    }
}

#[graphql_object(Context = RootContext)]
impl MutationRoot {
    fn add_tenant(&self, ctx: &RootContext, input: TenantInput) -> FieldResult<Tenant> {
        self.tenant_service.add_tenant(input)
    }

    fn add_user(&self, ctx: &RootContext, user: UserInput) -> FieldResult<User> {
        self.user_service.add_user(user)
    }

    fn login(&self, ctx: &RootContext, input: LoginInput) -> FieldResult<TokenResponse> {
        self.user_service.login(input)
    }

    fn create_policy(&self, ctx: RootContext, policy: PolicyInput) -> FieldResult<DBPolicy> {

    }

    fn add_permission_to_policy(&self, ctx: RootContext, policy_id: Uuid, permission: PermissionInput) -> FieldResult<DBPolicy> {

    }

    fn remove_permission_from_policy(&self, ctx: RootContext, policy_id: Uuid, permission: PermissionInput) -> FieldResult<DBPolicy> {

    }

    fn link_policy_to_user(&self, ctx: RootContext, policy_id: Uuid, user_id: Uuid) -> FieldResult<(DBUser, DBPolicy)> {

    }
}

pub struct RootContext {}

impl Context for RootContext {}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<RootContext>>;
