# osshkeys

[![Crates](https://img.shields.io/crates/v/osshkeys.svg)](https://crates.io/crates/osshkeys)
[![Docs](https://docs.rs/osshkeys/badge.svg)](https://docs.rs/osshkeys)

## Description
A Rust library to handle OpenSSH key and other common SSH key

The main function of this library is to read, write different formats of SSH keys.
Also, it provide the ability to generate a key, sign and verify data.

## Current Status
The library is still under development, so there are some functions that haven't implemented.
Some api may also change in the future.

## Example
```rust
#[macro_use]
extern crate hex_literal;
use osshkeys::{KeyPair, KeyType, Key as _, PublicPart as _, PrivatePart as _};
use osshkeys::keys::FingerprintHash;

fn main() {
    let keyfile = std::fs::read_to_string("assets/openssh_ed25519_enc").unwrap();
    let keypair = KeyPair::from_keystr(&keyfile, Some(b"12345678")).unwrap();

    // Get the public key
    let publickey = keypair.clone_public_key().unwrap();

    // Get the key type
    assert_eq!(keypair.keytype(), KeyType::ED25519);

    // Get the fingerprint
    assert_eq!(keypair.fingerprint(FingerprintHash::MD5).unwrap(), hex!("d29552b0c87d7ff1acb3c2229e783321"));

    // Sign some data
    const SOME_DATA: &[u8] = b"8Kn9PPQV";
    let sign = keypair.sign(SOME_DATA).unwrap();

    assert_eq!(sign.as_slice(), hex!("7206f04ef062ec35f8fb9f9e8a17ec023070ecf5f6e1021ea2af73137b1b832bba08766e5ad95fdca81af37b27898428f9a7dbeb044dd550afeb46efb94fe808").as_ref());
    assert!(publickey.verify(SOME_DATA, &sign).unwrap());
}
```

## Planning Features
- Core Features
    - Key Types
        - RSA
        - DSA
        - EcDSA
        - Ed25519
    - [x] Documentation
        - [x] Descriptions
        - [x] Examples
        - [ ] More Examples
    - [x] Key generation
    - [x] Public key formats
        - [x] Openssh
        - [ ] PEM
    - [x] Private keys
        - [x] PEM (Using OpenSSL)
        - [x] PEM (Encrypted) (Using OpenSSL)
        - [x] PKCS#8 (Using OpenSSL)
            - [x] Read
            - [ ] Write
        - [x] PKCS#8 (Encrypted) (Using OpenSSL)
            - [x] Read
            - [ ] Write
        - [x] Openssh v2
            - [x] Read
            - [ ] Write
        - [x] Openssh v2 (Encrypted)
            - [x] Read
            - [ ] Write
- Additional Features
    - [ ] Supporting XMSS keys
    - [ ] Supporting read/write Putty key format(.ppk)
    - [ ] Without using openssl (To become pure Rust library) (if there exists required cryptography crates being mature enough)

