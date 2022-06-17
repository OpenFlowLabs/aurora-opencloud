use crate::database::schema::user_confirmations;
use crate::database::schema::users;
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;
use thiserror::Error;
use ::pwhash::bcrypt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("Either username or email must be provided")]
    InvalidInput,
    #[error("Password does not match")]
    PasswordDoesNotMatch,
}

pub fn get_user(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    email_param: Option<String>,
    username_param: Option<String>,
) -> Result<DBUser> {
    if let Some(email_param) = email_param {
        Ok(users::table
            .filter(users::email.eq(email_param).and(users::tenant_id.eq(tenant_id_param)))
            .first::<DBUser>(&pool.get()?)?)
    } else if let Some(username_param) = username_param {
        Ok(users::table
            .filter(
                users::username
                    .eq(username_param)
                    .and(users::tenant_id.eq(tenant_id_param)),
            )
            .first::<DBUser>(&pool.get()?)?)
    } else {
        Err(UserRepoError::InvalidInput.into())
    }
}

pub fn list_users(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    limit: u64,
    offset: u64,
) -> Result<Vec<DBUser>> {
    let results: Vec<DBUser>;
    if offset != 0 && limit != 0 {
        results = users::table
            .filter(users::tenant_id.eq(tenant_id_param))
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<DBUser>(&pool.get()?)?;
    } else if limit != 0 {
        results = users::table
            .filter(users::tenant_id.eq(tenant_id_param))
            .limit(limit as i64)
            .load::<DBUser>(&pool.get()?)?;
    } else {
        results = users::table
            .filter(users::tenant_id.eq(tenant_id_param))
            .load::<DBUser>(&pool.get()?)?;
    }

    Ok(results)
}

pub fn create_user(pool: &PGPool, new_user: &NewUser) -> Result<(DBUser, String)> {
    let result: DBUser = diesel::insert_into(users::table)
        .values(new_user)
        .get_result(&pool.get()?)?;

    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    diesel::insert_into(user_confirmations::table)
        .values(&NewConfirmation {
            user_id: &result.id,
            tenant_id: &result.tenant_id,
            email: &result.email,
            token: &rand_string,
        })
        .execute(&pool.get()?)?;

    Ok((result, rand_string))
}

pub fn update_user(pool: &PGPool, user_id_param: &Uuid, tenant_id_param: &Uuid, username_param: Option<String>, email_param: Option<String>) -> Result<DBUser> {
    let target = users::table.filter(users::tenant_id.eq(tenant_id_param)).find(user_id_param);

    let result = if email_param.is_some() && username_param.is_none() {
        let new_email = email_param.ok_or(UserRepoError::InvalidInput)?;
        Ok(diesel::update(target).set(users::email.eq(new_email)).get_result::<DBUser>(&pool.get()?)?)
    } else if username_param.is_some() && email_param.is_none() {
        let new_username = username_param.ok_or(UserRepoError::InvalidInput)?;
        Ok(diesel::update(target).set(users::username.eq(new_username)).get_result::<DBUser>(&pool.get()?)?)
    } else if email_param.is_some() && username_param.is_some() {
        let new_email = email_param.ok_or(UserRepoError::InvalidInput)?;
        let new_username = username_param.ok_or(UserRepoError::InvalidInput)?;
        Ok(diesel::update(target).set((users::username.eq(new_username), users::email.eq(new_email))).get_result::<DBUser>(&pool.get()?)?)
    } else {
        Err(UserRepoError::InvalidInput)
    };

    Ok(result?)
}

pub fn delete_user(pool: &PGPool, user_id_param: &Uuid, tenant_id_param: &Uuid) -> Result<()> {
    let target = users::table.filter(users::tenant_id.eq(tenant_id_param)).find(user_id_param);
    diesel::delete(target).execute(&pool.get()?)?;
    Ok(())
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub tenant_id: Uuid,
    pub pwhash: String,
}

#[derive(Insertable)]
#[table_name = "user_confirmations"]
struct NewConfirmation<'a> {
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
    pub email_confirmed: bool,
}

impl DBUser {
    pub fn verify_pw(&self, pw_to_check: &str) -> bool {
        bcrypt::verify(pw_to_check, &self.pwhash)
    }

    pub fn verify_pw_result(&self, pw_to_check: &str) -> Result<bool, UserRepoError> {
        if !self.verify_pw(pw_to_check) {
            return Err(UserRepoError::PasswordDoesNotMatch);
        }
        Ok(true)
    }
}