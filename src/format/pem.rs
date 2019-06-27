use crate::error::*;
use crate::keys::*;
use aes::{Aes128, Aes192, Aes256};
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use des::{Des, TdesEde3};
use nom_pem::{Block as PemBlock, HeaderEntry, ProcTypeType, RFC1423Algorithm};
use openssl::pkey::PKey;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;
type Aes192Cbc = Cbc<Aes192, Pkcs7>;
type Aes256Cbc = Cbc<Aes256, Pkcs7>;
type DesCbc = Cbc<Des, Pkcs7>;
type DesEde3Cbc = Cbc<TdesEde3, Pkcs7>;

//TODO: Not to use openssl to parse pem file in the future
pub fn parse_pem_privkey(pem: &[u8], passphrase: Option<&[u8]>) -> OsshResult<KeyPair> {
    let pkey = if let Some(passphrase) = passphrase {
        PKey::private_key_from_pem_passphrase(pem, passphrase)
            .map_err(|_| ErrorKind::IncorrectPass)?
    } else {
        PKey::private_key_from_pem(pem)?
    };

    Ok(KeyPair::from_ossl_pkey(&pkey)?)
}

//TODO: Not to use openssl to parse pem file in the future
pub fn stringify_pem_privkey(keypair: &KeyPair, passphrase: Option<&[u8]>) -> OsshResult<String> {
    let pem = if let Some(passphrase) = passphrase {
        let cipher = openssl::symm::Cipher::aes_128_cbc();
        match &keypair.key {
            KeyPairType::RSA(key) => key
                .ossl_rsa()
                .private_key_to_pem_passphrase(cipher, passphrase)?,
            KeyPairType::DSA(key) => unimplemented!(), //FIXME: openssl crate not implement it!!! //key.ossl_dsa().private_key_to_pem_passphrase(cipher, passphrase)?,
            KeyPairType::ECDSA(key) => key
                .ossl_ec()
                .private_key_to_pem_passphrase(cipher, passphrase)?,
            _ => return Err(ErrorKind::UnsupportType.into()),
        }
    } else {
        match &keypair.key {
            KeyPairType::RSA(key) => key.ossl_rsa().private_key_to_pem()?,
            KeyPairType::DSA(key) => unimplemented!(), //FIXME: openssl crate not implement it!!! //key.ossl_dsa().private_key_to_pem()?,
            KeyPairType::ECDSA(key) => key.ossl_ec().private_key_to_pem()?,
            _ => return Err(ErrorKind::UnsupportType.into()),
        }
    };

    Ok(String::from_utf8(pem).map_err(|e| Error::with_failure(ErrorKind::InvalidPemFormat, e))?)
}

pub fn parse_keyfile(pem: &[u8], passphrase: Option<&[u8]>) -> OsshResult<KeyPair> {
    let pemdata = nom_pem::decode_block(pem)?;

    match pemdata.block_type {
        "OPENSSH PRIVATE KEY" => {
            // Openssh format
            unimplemented!()
        }
        "PRIVATE KEY" => {
            // PKCS#8 format
            unimplemented!()
        }
        "DSA PRIVATE KEY" => {
            // Openssl DSA Key
            unimplemented!()
        }
        "RSA PRIVATE KEY" => {
            // Openssl RSA Key
            unimplemented!()
        }
        "EC PRIVATE KEY" => {
            // Openssl EC Key
            unimplemented!()
        }
        _ => return Err(ErrorKind::UnsupportType.into()),
    }

    unimplemented!()
}

fn pem_decrypt(pemblock: &nom_pem::Block, passphrase: Option<&[u8]>) -> OsshResult<Vec<u8>> {
    let mut encrypted = false;
    for entry in &pemblock.headers {
        if let HeaderEntry::ProcType(_, proctype) = entry {
            if *proctype == ProcTypeType::ENCRYPTED {
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
                    //TODO: key derived function not implemented yet!!!
                    decrypted = Some(
                        match algo {
                            RFC1423Algorithm::DES_CBC => {
                                DesCbc::new_var(&pass, &iv)?.decrypt_vec(&pemblock.data)
                            }
                            RFC1423Algorithm::DES_EDE3_CBC => {
                                DesEde3Cbc::new_var(&pass, &iv)?.decrypt_vec(&pemblock.data)
                            }
                            RFC1423Algorithm::AES_128_CBC => {
                                Aes128Cbc::new_var(&pass, &iv)?.decrypt_vec(&pemblock.data)
                            }
                            RFC1423Algorithm::AES_192_CBC => {
                                Aes192Cbc::new_var(&pass, &iv)?.decrypt_vec(&pemblock.data)
                            }
                            RFC1423Algorithm::AES_256_CBC => {
                                Aes256Cbc::new_var(&pass, &iv)?.decrypt_vec(&pemblock.data)
                            }
                        }
                        .map_err(|_| ErrorKind::IncorrectPass)?,
                    );
                } else {
                    return Err(ErrorKind::IncorrectPass.into());
                }
                break;
            }
        }
        if let Some(data) = decrypted {
            Ok(data)
        } else {
            return Err(ErrorKind::InvalidPemFormat.into());
        }
    } else {
        Ok(pemblock.data.clone())
    }
}
