use uuid::Uuid;
use juniper::{GraphQLObject, GraphQLInputObject, FieldResult, FieldError, graphql_value};
use crate::adapters::database::user::{UserRepository, NewUser, DBUser, NewPermission};
use pwhash::bcrypt;
use std::option::Option::Some;
use josekit::jws::JwsHeader;
use josekit::jwt::JwtPayload;
use josekit::{jwt, Value, Map, Number};
use serde_json::json;
use std::time::{Duration, SystemTime};
use josekit::jws::alg::eddsa::EddsaJwsSigner;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use crate::adapters::database::common::PGPool;

#[derive(Clone)]
pub struct UserService {
    user_repository: Box<UserRepository>,
    issuer_base: String,
    signer: EddsaJwsSigner,
}

impl UserService {
    pub fn new(pool: PGPool, issuer_base: String, signer: EddsaJwsSigner) -> Self {
        UserService {
            user_repository: Box::new(UserRepository::new(pool)),
            issuer_base,
            signer,
        }
    }

    pub fn get_user(&self, tenant: Uuid, email: Option<String>, name: Option<String>) -> FieldResult<Option<User>> {
        match email {
            Some(e) => {
                let result = self.user_repository.get_user_by_email(tenant, e)?;
                return Ok(Some(User::from(result)));
            }
            _ => {}
        }
        match name {
            Some(n) => {
                let result = self.user_repository.get_user_by_name(tenant, n)?;
                return Ok(Some(User::from(result)));
            }
            _ => {}
        }
        Err(FieldError::new(
            "either username or email must be provided",
            graphql_value!({ "bad_request": "not enough arguments" })
        ))
    }

    pub fn users(&self, tenant: Uuid, limit: usize, offset: usize) -> FieldResult<Option<Vec<User>>> {
        let result = self.user_repository.users(tenant, limit, offset)?;
        let ret_vec = result.into_iter().map(|db_user| User::from(db_user)).collect();
        Ok(Some(ret_vec))
    }

    pub fn add_user(&self, input: UserInput) -> FieldResult<User> {
        let result = self.user_repository.add_user(&NewUser{
            username: &input.username,
            tenant_id: &input.tenant,
            email: &input.email,
            pwhash: &bcrypt::hash(input.password)?
        })?;
        //TODO send mail to user
        Ok(User::from(result.0))
    }

    pub fn login(&self, input: LoginInput) -> FieldResult<TokenResponse> {
        if let Some(email) = input.email {
            self.login_db_user(self.user_repository.get_user_by_email(input.tenant, email)?, &input.password, input.remember)
        } else if let Some(username) = input.username {
            self.login_db_user(self.user_repository.get_user_by_name(input.tenant, username)?, &input.password, input.remember)
        } else {
            Err(FieldError::new(
                "either username or email must be provided",
                graphql_value!({ "bad_request": "not enough arguments" })
            ))
        }
    }

    pub fn update_user(&self, input: UserInput) -> FieldResult<User> {

    }

    pub fn delete_user(&self, user_id: Uuid) -> FieldResult<User> {

    }

    pub fn attach_policy_to_user(&self, user_id: Uuid, policy_id: Uuid) -> FieldResult<User> {

    }

    pub fn dettach_policy_from_user(&self, user_id: Uuid, policy_id: Uuid) -> FieldResult<User> {

    }

    fn login_db_user(&self, user: DBUser, password: &str, remember: bool) -> FieldResult<TokenResponse> {
        if !user.verify_pw(password) {
            return Err(FieldError::new(
                "Password not correct",
                graphql_value!({ "bad_request": "wrong password" })
            ));
        }

        let perms = self.user_repository.get_permissions(user.id, user.tenant_id)?;
        let auth_token = {
            let mut header = JwsHeader::new();
            header.set_token_type("JWT");
            header.set_algorithm("ED25519");
            let payload = JwtPayload::from_map(AuthToken::new(&user, self.issuer_base.clone(), perms))?;
            jwt::encode_with_signer(&payload, &header, &self.signer)?
        };

        let refresh_token = {
            if remember {
                let mut header = JwsHeader::new();
                header.set_token_type("JWT");
                header.set_algorithm("ED25519");
                let payload = JwtPayload::from_map(RefreshToken::new(&user, self.issuer_base.clone()))?;
                Some(jwt::encode_with_signer(&payload, &header, &self.signer)?)
            } else {
                None
            }

        };

        Ok(TokenResponse{
            auth_token,
            refresh_token,
        })
    }
}

#[derive(GraphQLInputObject)]
pub struct UserInput {
    username: String,
    password: String,
    tenant: Uuid,
    email: String,
}

#[derive(GraphQLObject, Default, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub tenant_id: Uuid,
    pub email: String,
    pub email_confirmed: bool,
    pub policies: Vec<String>,
    pub effective_permissions: Vec<String>,
}

#[derive(GraphQLInputObject)]
pub struct LoginInput {
    pub tenant: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub remember: bool,
}

#[derive(GraphQLObject, Default, Clone)]
pub struct TokenResponse {
    pub auth_token: String,
    pub refresh_token: Option<String>,
}

impl From<DBUser> for User {
    fn from(db_user: DBUser) -> Self {
        User{
            id: db_user.id,
            username: db_user.username,
            tenant_id: db_user.tenant_id,
            email: db_user.email,
            email_confirmed: db_user.email_confirmed
        }
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
    fn new(db_user: &DBUser, issuer_base: String, permissions: Vec<String>) -> Self {
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
    fn new(db_user: &DBUser, issuer_base: String) -> Self {
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