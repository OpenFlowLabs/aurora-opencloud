use diesel::prelude::*;
use juniper::{FieldResult, FieldError};
use uuid::Uuid;
use crate::adapters::database::schema::users;
use crate::adapters::database::schema::user_confirmations;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::adapters::database::schema::permissions;
use juniper::graphql_value;
use pwhash::bcrypt;
use crate::adapters::database::common::PGPool;

#[derive(Clone)]
pub struct UserRepository {
    pool: PGPool,
}

impl UserRepository {
    pub fn get_user_by_name(&self, tenant: Uuid, username: String) -> FieldResult<DBUser> {
        let user = users::table
            .filter(users::username.eq(username).and(users::tenant_id.eq(tenant)))
            .first::<DBUser>(&self.pool.get()?)?;
        Ok(user)
    }

    pub fn get_user_by_email(&self, tenant: Uuid, email: String) -> FieldResult<DBUser> {
        let user = users::table
            .filter(users::email.eq(email).and(users::tenant_id.eq(tenant)))
            .first::<DBUser>(&self.pool.get()?)?;
        Ok(user)
    }

    pub fn users(&self, tenant: Uuid, limit: usize, offset: usize) -> FieldResult<Vec<DBUser>> {
        let results: Vec<DBUser>;
        if offset != 0 && limit != 0 {
            results = users::table
                .filter(users::tenant_id.eq(tenant))
                .limit(limit as i64)
                .offset(offset as i64)
                .load::<DBUser>(&self.pool.get()?)?;
        } else if limit != 0 {
            results = users::table
                .filter(users::tenant_id.eq(tenant))
                .limit(limit as i64)
                .load::<DBUser>(&self.pool.get()?)?;
        } else {
            results = users::table
                .filter(users::tenant_id.eq(tenant))
                .load::<DBUser>(&self.pool.get()?)?;
        }

        Ok(results)
    }

    pub fn add_user(&self, new_user: &NewUser) -> FieldResult<(DBUser, String)> {
        let result: DBUser = diesel::insert_into(users::table)
            .values(new_user)
            .get_result(&self.pool.get()?)?;

        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        diesel::insert_into(user_confirmations::table)
            .values(&NewConfirmation{
                user_id: &result.id,
                tenant_id: &result.tenant_id,
                email: &result.email,
                token: &rand_string,
            })
            .execute(&self.pool.get()?)?;

        Ok((result, rand_string))
    }

    pub fn new(pool: PGPool) -> Self {
        UserRepository{
            pool
        }
    }
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a String,
    pub email: &'a String,
    pub tenant_id: &'a Uuid,
    pub pwhash: &'a String
}

#[derive(Insertable)]
#[table_name="user_confirmations"]
pub struct NewConfirmation<'a> {
    pub user_id: &'a Uuid,
    pub tenant_id: &'a Uuid,
    pub token: &'a String,
    pub email: &'a String,
}

#[derive(Queryable, Default, Clone)]
pub struct DBUser {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub username: String,
    pwhash: String,
    pub email: String,
    pub email_confirmed: bool
}

#[derive(Insertable)]
#[table_name="permissions"]
pub struct NewPermission<'a> {
    pub user_id: &'a Uuid,
    pub tenant_id: &'a Uuid,
    pub permission: &'a String,
}

impl DBUser {
    pub fn verify_pw(&self, pw_to_check: &str) -> bool {
        bcrypt::verify(pw_to_check, &self.pwhash)
    }
}