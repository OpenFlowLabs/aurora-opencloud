// Temporary disable unused codes warnings in this file
#![allow(dead_code)]

use crate::cipher::*;
use crate::error::*;
use crate::keys::{rsa::*, *};
use digest::DynDigest;
use nom_pem::{Block as PemBlock, HeaderEntry, ProcTypeType, RFC1423Algorithm};
use openssl::{
    pkey::{PKey, Public},
    rsa::Rsa,
};
use zeroize::Zeroize;

const MAX_KEY_LEN: usize = 64;

//TODO: Not to depend on openssl to parse pem file in the future
pub fn parse_pem_privkey(pem: &[u8], passphrase: Option<&str>) -> OsshResult<KeyPair> {
    let pkey = if let Some(passphrase) = passphrase {
        PKey::private_key_from_pem_passphrase(pem, passphrase.as_bytes())
            .map_err(|_| ErrorKind::IncorrectPass)?
    } else {
        PKey::private_key_from_pem(pem)?
    };

    KeyPair::from_ossl_pkey(&pkey)
}

//TODO: Not to depend on openssl to parse pem file in the future
pub fn stringify_pem_privkey(keypair: &KeyPair, passphrase: Option<&str>) -> OsshResult<String> {
    let pem = if let Some(passphrase) = passphrase {
        // TODO: Allow for cipher selection
        let cipher = openssl::symm::Cipher::aes_128_cbc();
        let passphrase = passphrase.as_bytes();
        match &keypair.key {
            KeyPairType::RSA(key) => key
                .ossl_rsa()
                .private_key_to_pem_passphrase(cipher, passphrase)?,
            KeyPairType::DSA(key) => key
                .ossl_dsa()
                .private_key_to_pem_passphrase(cipher, passphrase)?,
            KeyPairType::ECDSA(key) => key
                .ossl_ec()
                .private_key_to_pem_passphrase(cipher, passphrase)?,
            KeyPairType::ED25519(key) => key
                .ossl_pkey()?
                .private_key_to_pem_pkcs8_passphrase(cipher, passphrase)?,
        }
    } else {
        match &keypair.key {
            KeyPairType::RSA(key) => key.ossl_rsa().private_key_to_pem()?,
            KeyPairType::DSA(key) => key.ossl_dsa().private_key_to_pem()?,
            KeyPairType::ECDSA(key) => key.ossl_ec().private_key_to_pem()?,
            KeyPairType::ED25519(key) => key.ossl_pkey()?.private_key_to_pem_pkcs8()?,
        }
    };

    String::from_utf8(pem).map_err(|e| Error::with_error(ErrorKind::InvalidPemFormat, e))
}

pub fn parse_pem_pubkey(pem: &[u8]) -> OsshResult<PublicKey> {
    if pem.starts_with(b"-----BEGIN RSA PUBLIC KEY-----") {
        let rsa = Rsa::<Public>::public_key_from_pem_pkcs1(pem)?;
        let rsapubkey = RsaPublicKey::from_ossl_rsa(rsa, RsaSignature::SHA1)?;
        Ok(rsapubkey.into())
    } else {
        let pkey = PKey::public_key_from_pem(pem)?;
        Ok(PublicKey::from_ossl_pkey(&pkey)?)
    }
}

pub fn stringify_pem_pubkey(pubkey: &PublicKey) -> OsshResult<String> {
    let pem = match &pubkey.key {
        PublicKeyType::RSA(key) => key.ossl_rsa().public_key_to_pem_pkcs1()?,
        PublicKeyType::DSA(key) => key.ossl_pkey()?.public_key_to_pem()?,
        PublicKeyType::ECDSA(key) => key.ossl_pkey()?.public_key_to_pem()?,
        PublicKeyType::ED25519(key) => key.ossl_pkey()?.public_key_to_pem()?,
    };

    String::from_utf8(pem).map_err(|e| Error::with_error(ErrorKind::InvalidPemFormat, e))
}

/// Self experimental implementation for decrypting OpenSSL PEM format
fn pem_decrypt(pemblock: &PemBlock, passphrase: Option<&[u8]>) -> OsshResult<Vec<u8>> {
    let mut encrypted = false;
    for entry in &pemblock.headers {
        if let HeaderEntry::ProcType(ver, proctype) = entry {
            if *proctype == ProcTypeType::ENCRYPTED && *ver == 4 {
                encrypted = true;
            } else {
                return Err(ErrorKind::UnsupportType.into());
            }
        }
    }
    if encrypted {
        let mut decrypted = None;
        for entry in &pemblock.headers {
            if let HeaderEntry::DEKInfo(algo, iv) = entry {
                if let Some(pass) = passphrase {
                    let ciph = match algo {
                        RFC1423Algorithm::DES_EDE3_CBC => Cipher::TDes_Cbc,
                        RFC1423Algorithm::AES_128_CBC => Cipher::Aes128_Cbc,
                        RFC1423Algorithm::AES_192_CBC => Cipher::Aes192_Cbc,
                        RFC1423Algorithm::AES_256_CBC => Cipher::Aes256_Cbc,
                        _ => return Err(ErrorKind::UnsupportCipher.into()),
                    };
                    let key = openssl_kdf(
                        pass,
                        iv.as_slice().try_into()?,
                        &mut md5::Md5::default(),
                        ciph.key_len(),
                        1,
                    )?;
                    decrypted = Some(ciph.decrypt(&pemblock.data, &key, iv)?);
                } else {
                    return Err(ErrorKind::IncorrectPass.into());
                }
                break;
            }
        }
        if let Some(data) = decrypted {
            Ok(data)
        } else {
            Err(ErrorKind::InvalidPemFormat.into())
        }
    } else {
        Ok(pemblock.data.clone())
    }
}

/// Self experimental implementation for OpenSSL kdf
///
/// From OpenSSL EVP_BytesToKey()
fn openssl_kdf(
    data: &[u8],
    salt: &[u8; 8],
    digest: &mut dyn DynDigest,
    keylen: usize,
    iter: usize,
) -> OsshResult<Vec<u8>> {
    if keylen > MAX_KEY_LEN {
        return Err(ErrorKind::InvalidKeySize.into());
    }

    let mut key: Vec<u8> = Vec::with_capacity(keylen);
    let mut dig: Box<[u8]> = Box::default();

    let mut first = true;
    digest.reset();
    while key.len() < keylen {
        if !first {
            digest.update(&dig);
        }
        digest.update(data);
        digest.update(salt);
        dig = digest.finalize_reset();

        for _ in 1..iter {
            digest.update(&dig);
            dig = digest.finalize_reset();
        }

        for byte in dig.as_ref() {
            if key.len() < keylen {
                key.push(*byte);
            }
        }

        first = false;
    }

    dig.zeroize();
    Ok(key)
}
