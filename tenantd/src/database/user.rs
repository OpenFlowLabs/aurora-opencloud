use std::time::{Duration, SystemTime};
use crate::database::schema::user_confirmations;
use crate::database::schema::users;
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;
use failure::Fail;
use ::pwhash::bcrypt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Map, Number, Value};
use uuid::Uuid;

#[derive(Fail, Debug)]
pub enum UserRepoError {
    #[fail(display = "An unknown error has occurred.")]
    Unknown,
    #[fail(display = "Either username or email must be provided")]
    InvalidInput,
    #[fail(display = "Password does not match")]
    PasswordDoesNotMatch,
}

pub fn get_user(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    email_param: Option<String>,
    username_param: Option<String>,
) -> Fallible<DBUser> {
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
) -> Fallible<Vec<DBUser>> {
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

pub fn create_user(pool: &PGPool, new_user: &NewUser) -> Fallible<(DBUser, String)> {
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

pub fn update_user(pool: &PGPool, user_id_param: &Uuid, tenant_id_param: &Uuid, username_param: Option<String>, email_param: Option<String>) -> Fallible<DBUser> {
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

pub fn delete_user(pool: &PGPool, user_id_param: &Uuid, tenant_id_param: &Uuid) -> Fallible<()> {
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

pub struct AuthToken {
    // The URL for the API issueing the token plus tenant_id
    issuer: String,

    // UUID of the tennant
    tennant_id: String,

    // UUID of the user
    subject: String,

    // Array of permission strings
    permissions: Vec<String>,
}

impl AuthToken{
    pub fn new(db_user: &DBUser, issuer_base: String, permissions: Vec<String>) -> Self {
        AuthToken{
            issuer: issuer_base + "/" + db_user.tenant_id.to_string().as_str(),
            subject: db_user.id.to_string(),
            tennant_id: db_user.tenant_id.to_string(),
            permissions,
        }
    }
}

impl Into<Map<String, Value>> for AuthToken {
    fn into(self) -> Map<String, Value> {
        let mut map = Map::new();
        let issue_time = SystemTime::now();
        if let Some(expiration) = issue_time.checked_add(Duration::from_secs(3600)) {
            map.insert("exp".to_owned(), time_to_value(&expiration));
        }
        if let Some(not_before) = issue_time.checked_sub(Duration::from_secs(120)) {
            map.insert("nbf".to_owned(), time_to_value(&not_before));
        }
        map.insert("iat".to_owned(), time_to_value(&issue_time));

        map.insert("iss".to_owned(), json!(self.issuer));
        map.insert("sub".to_owned(), json!(self.subject));
        map.insert("perms".to_owned(), json!(self.permissions));
        map.insert("tennant".to_owned(), json!(self.tennant_id));
        map
    }
}

pub struct RefreshToken {
    // The URL for the API issueing the token plus tenant_id
    issuer: String,

    // UUID of the tennant
    tennant_id: String,

    // UUID of the user
    subject: String,
}

fn time_to_value(value: &SystemTime) -> Value {
    Value::Number(Number::from(
        value
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    ))
}

impl RefreshToken{
    pub fn new(db_user: &DBUser, issuer_base: String) -> Self {
        RefreshToken{
            issuer: issuer_base + "/" + db_user.tenant_id.to_string().as_str(),
            subject: db_user.id.to_string(),
            tennant_id: db_user.tenant_id.to_string(),
        }
    }
}

impl Into<Map<String, Value>> for RefreshToken {
    fn into(self) -> Map<String, Value> {
        let mut map = Map::new();
        let issue_time = SystemTime::now();
        if let Some(expiration) = issue_time.checked_add(Duration::from_secs(259200)) {
            map.insert("exp".to_owned(), time_to_value(&expiration));
        }
        if let Some(not_before) = issue_time.checked_sub(Duration::from_secs(120)) {
            map.insert("nbf".to_owned(), time_to_value(&not_before));
        }
        map.insert("iat".to_owned(), time_to_value(&issue_time));

        map.insert("iss".to_owned(), json!(self.issuer));
        map.insert("sub".to_owned(), json!(self.subject));
        map.insert("tennant".to_owned(), json!(self.tennant_id));
        map
    }
}