use super::{Key, PrivKey, PubKey};
use crate::error::{Error, ErrorKind, OsshResult};
use crate::format::ossh_pubkey::*;
#[rustfmt::skip]
use ed25519_dalek::{
    Keypair as DalekKeypair,
    PublicKey as DalekPublicKey,
    SecretKey as DalekSecretKey,
    Signature,
    PUBLIC_KEY_LENGTH,
};
use rand::rngs::OsRng;
use std::fmt;

pub const ED25519_NAME: &str = "ssh-ed25519";

#[derive(Debug, Clone)]
pub struct Ed25519PublicKey {
    key: DalekPublicKey,
}

impl Ed25519PublicKey {
    pub fn new(key: &[u8; PUBLIC_KEY_LENGTH]) -> Result<Self, ed25519_dalek::SignatureError> {
        Ok(Self {
            key: DalekPublicKey::from_bytes(key)?,
        })
    }
}

impl Key for Ed25519PublicKey {
    fn size(&self) -> usize {
        256
    }

    fn keyname(&self) -> &'static str {
        ED25519_NAME
    }
}

impl PubKey for Ed25519PublicKey {
    fn blob(&self) -> Result<Vec<u8>, Error> {
        Ok(encode_ed25519_pubkey(&self.key)?)
    }

    fn verify(&self, data: &[u8], sig: &[u8]) -> Result<bool, Error> {
        let ed25519_sig = Signature::from_bytes(sig)?;
        Ok(self.key.verify(data, &ed25519_sig).is_ok())
    }
}

impl PartialEq for Ed25519PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl fmt::Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&stringify_ossh_pubkey(self, None).unwrap())
    }
}

pub struct Ed25519KeyPair {
    key: DalekKeypair,
}

impl Key for Ed25519KeyPair {
    fn size(&self) -> usize {
        256
    }

    fn keyname(&self) -> &'static str {
        ED25519_NAME
    }
}

impl Ed25519KeyPair {
    pub fn generate(bits: usize) -> OsshResult<Self> {
        if bits != 0 && bits != 256 {
            return Err(Error::from_kind(ErrorKind::InvalidKeySize));
        }

        let mut rng = OsRng::new().map_err(|e| Error::with_failure(ErrorKind::Ed25519Error, e))?;
        Ok(Ed25519KeyPair {
            key: DalekKeypair::generate(&mut rng),
        })
    }

    pub(crate) fn from_bytes(pk: &[u8], sk: &[u8]) -> OsshResult<Self> {
        Ok(Ed25519KeyPair {
            key: DalekKeypair {
                public: DalekPublicKey::from_bytes(pk)?,
                secret: DalekSecretKey::from_bytes(sk)?,
            },
        })
    }

    pub fn clone_public_key(&self) -> Result<Ed25519PublicKey, Error> {
        Ok(Ed25519PublicKey {
            key: self.key.public.clone(),
        })
    }
}

impl PubKey for Ed25519KeyPair {
    fn blob(&self) -> Result<Vec<u8>, Error> {
        Ok(encode_ed25519_pubkey(&self.key.public)?)
    }

    fn verify(&self, data: &[u8], sig: &[u8]) -> Result<bool, Error> {
        let ed25519_sig = Signature::from_bytes(sig)?;
        Ok(self.key.verify(data, &ed25519_sig).is_ok())
    }
}

impl PrivKey for Ed25519KeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(self.key.sign(data).to_bytes().to_vec())
    }
}

#[allow(non_upper_case_globals)]
#[cfg(test)]
mod test {
    use super::*;

    const pub_str: &'static str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMoWBluPErgKhNja3lHEf7ie6AVzR24mPRd742xEYodC";
    const pub_key: [u8; 0x20] = [
        0xca, 0x16, 0x06, 0x5b, 0x8f, 0x12, 0xb8, 0x0a, 0x84, 0xd8, 0xda, 0xde, 0x51, 0xc4, 0x7f,
        0xb8, 0x9e, 0xe8, 0x05, 0x73, 0x47, 0x6e, 0x26, 0x3d, 0x17, 0x7b, 0xe3, 0x6c, 0x44, 0x62,
        0x87, 0x42,
    ];

    fn get_test_pubkey() -> Result<Ed25519PublicKey, Error> {
        Ok(Ed25519PublicKey::new(&pub_key)?)
    }

    #[test]
    fn ed25519_publickey_serialize() {
        let key = get_test_pubkey().unwrap();
        assert_eq!(key.to_string(), String::from(pub_str));
    }

    #[test]
    fn ed25519_publickey_size() {
        let key = get_test_pubkey().unwrap();
        assert_eq!(key.size(), 256);
    }
}
