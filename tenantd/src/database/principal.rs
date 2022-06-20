use crate::database::schema::{principals, principals_pks, public_keys, user_confirmations};
use crate::database::PGPool;
use common::*;
use diesel::prelude::*;
use osshkeys::{keys::FingerprintHash, PublicKey as SSHPublicKey, PublicParts};
use pasetors::keys::AsymmetricPublicKey;
use pasetors::paserk::FormatAsPaserk;
use pasetors::version4::V4;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum PrincipalRepoError {
    #[error("Either name or email must be provided")]
    InvalidInput,
}

pub fn get_principal(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    email_param: Option<String>,
    name_param: Option<String>,
) -> Result<Principal> {
    if let Some(email_param) = email_param {
        Ok(principals::table
            .filter(
                principals::email
                    .eq(email_param)
                    .and(principals::tenant_id.eq(tenant_id_param)),
            )
            .first::<Principal>(&pool.get()?)?)
    } else if let Some(name_param) = name_param {
        Ok(principals::table
            .filter(
                principals::p_name
                    .eq(name_param)
                    .and(principals::tenant_id.eq(tenant_id_param)),
            )
            .first::<Principal>(&pool.get()?)?)
    } else {
        bail!(PrincipalRepoError::InvalidInput)
    }
}

pub fn get_principal_by_id(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    principal_id: &Uuid
) -> Result<Principal> {
    Ok(principals::table
        .filter(
            principals::id
                .eq(principal_id)
                .and(principals::tenant_id.eq(tenant_id_param)),
        )
        .first::<Principal>(&pool.get()?)?)
}

pub fn get_principal_with_key(pool: &PGPool, fingerprint: &str) -> Result<PrincipalWithKey> {
    public_keys::table
        .inner_join(principals_pks::table.inner_join(principals::table))
        .filter(public_keys::fingerprint.eq(fingerprint))
        .select((
            principals::columns::id,
            principals::columns::tenant_id,
            principals::columns::p_name,
            principals::columns::email,
            principals::columns::email_confirmed,
            public_keys::columns::fingerprint,
            public_keys::columns::public_key_paserk,
        ))
        .first::<PrincipalWithKey>(&pool.get()?)
        .map_err(|err| anyhow!("{}", err))
}

pub fn list_principals_of_tenant(
    pool: &PGPool,
    tenant_id_param: &Uuid,
    limit: u64,
    offset: u64,
) -> Result<Vec<Principal>> {
    let results: Vec<Principal>;
    if offset != 0 && limit != 0 {
        results = principals::table
            .filter(principals::tenant_id.eq(tenant_id_param))
            .limit(limit as i64)
            .offset(offset as i64)
            .load::<Principal>(&pool.get()?)?;
    } else if limit != 0 {
        results = principals::table
            .filter(principals::tenant_id.eq(tenant_id_param))
            .limit(limit as i64)
            .load::<Principal>(&pool.get()?)?;
    } else {
        results = principals::table
            .filter(principals::tenant_id.eq(tenant_id_param))
            .load::<Principal>(&pool.get()?)?;
    }

    Ok(results)
}

pub fn create_principal(
    pool: &PGPool,
    input: &NewPrincipalInput,
    mail_token: Option<String>,
    pub_keys: Vec<String>,
) -> Result<Principal> {
    if pub_keys.is_empty() {
        bail!("no public key provided")
    }

    let conn = &pool.get()?;
    trace!("Starting Transaction");
    conn.build_transaction().read_write().run(move || {
        trace!("Inserting Principal records");
        let principal: Principal = diesel::insert_into(principals::table)
            .values(input)
            .get_result(conn)?;

        if let Some(mail_token) = mail_token {
            trace!("Inserting email token for verification");
            let mail = if let Some(ref email) = principal.email {
                email
            } else {
                bail!(
                    "tried to create mail confirmation for principal {} but email is empty",
                    &principal.p_name
                )
            };

            let confirmation = NewConfirmation {
                p_id: &principal.id,
                tenant_id: &principal.tenant_id,
                token: &mail_token,
                email: mail,
            };

            diesel::insert_into(user_confirmations::table)
                .values(&confirmation)
                .execute(conn)?;
        }


        for key in pub_keys {
            trace!("Inserting Public key for Authentification");
            let public_key = public_key_from_keystring(&key)?;
            let fingerprint = public_key.fingerprint.clone();

            diesel::insert_into(public_keys::table)
                .values(public_key)
                .execute(conn)?;

            diesel::insert_into(principals_pks::table)
                .values(NewKeyPrincipalLinkInput {
                    p_id: &principal.id,
                    fingerprint: &fingerprint,
                })
                .execute(conn)?;
        }

        Ok(principal)
    })
}

pub fn add_ssh_key_to_principal(pool: &PGPool, principal_id: &Uuid, key: &str) -> Result<()> {
    let conn = &pool.get()?;

    conn.build_transaction().read_write().run(move || {
        let public_key = public_key_from_keystring(key)?;
        let fingerprint = public_key.fingerprint.clone();

        diesel::insert_into(public_keys::table)
            .values(public_key)
            .execute(conn)?;

        diesel::insert_into(principals_pks::table)
            .values(NewKeyPrincipalLinkInput {
                p_id: principal_id,
                fingerprint: &fingerprint,
            })
            .execute(conn)?;

        Ok(())
    })
}

pub fn remove_ssh_key_from_principal(
    pool: &PGPool,
    principal_id: &Uuid,
    fingerprint: &str,
) -> Result<()> {
    let conn = &pool.get()?;

    conn.build_transaction().read_write().run(move || {
        let public_key_target = principals_pks::table
            .filter(principals_pks::fingerprint.eq(fingerprint))
            .filter(principals_pks::p_id.eq(principal_id));
        diesel::delete(public_key_target).execute(conn)?;

        let public_key_target = public_keys::table.filter(public_keys::fingerprint.eq(fingerprint));
        diesel::delete(public_key_target).execute(conn)?;

        Ok(())
    })
}

pub fn delete_principal(pool: &PGPool, user_id_param: &Uuid, tenant_id_param: &Uuid) -> Result<()> {
    let target = principals::table
        .filter(principals::tenant_id.eq(tenant_id_param))
        .find(user_id_param);
    diesel::delete(target).execute(&pool.get()?)?;
    Ok(())
}

pub(crate) fn public_key_from_keystring(key_string: &str) -> Result<PublicKey> {
    trace!("Parsing OpenSSH Public Key");
    let ossh_key = SSHPublicKey::from_keystr(key_string)?;

    trace!("Writing PEM Key");
    let pem_key_string = ossh_key.serialize_pem()?;

    trace!("Creating PASERK formated key");
    let ossl_pem_key = openssl::pkey::PKey::public_key_from_pem(pem_key_string.as_bytes())?;
    let parsed_key = AsymmetricPublicKey::<V4>::from(&ossl_pem_key.raw_public_key()?)?;
    let mut paserk = String::new();
    parsed_key.fmt(&mut paserk)?;

    trace!("Getting Fingerprint");
    let fingerprint = hex::encode(ossh_key.fingerprint(FingerprintHash::SHA256)?);

    Ok(PublicKey {
        fingerprint,
        public_key: ossh_key.serialize()?,
        public_key_pem: pem_key_string,
        public_key_paserk: paserk,
    })
}

#[derive(Insertable)]
#[table_name = "principals_pks"]
struct NewKeyPrincipalLinkInput<'a> {
    p_id: &'a Uuid,
    fingerprint: &'a str,
}

#[derive(Insertable)]
#[table_name = "public_keys"]
pub(crate) struct PublicKey {
    fingerprint: String,
    public_key: String,
    public_key_pem: String,
    public_key_paserk: String,
}

#[derive(Insertable)]
#[table_name = "principals"]
pub struct NewPrincipalInput {
    pub p_name: String,
    pub email: Option<String>,
    pub tenant_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "user_confirmations"]
struct NewConfirmation<'a> {
    pub p_id: &'a Uuid,
    pub tenant_id: &'a Uuid,
    pub token: &'a str,
    pub email: &'a str,
}

#[derive(Queryable, Default, Clone)]
pub struct Principal {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub p_name: String,
    pub email: Option<String>,
    pub email_confirmed: Option<bool>,
}

#[derive(Queryable, Default, Clone)]
pub struct PrincipalWithKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub p_name: String,
    pub email: Option<String>,
    pub email_confirmed: Option<bool>,
    pub public_key_paserk: String,
    pub fingerprint: String,
}
