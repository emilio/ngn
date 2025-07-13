//! Ed25519 utils, signing and verification.

use bincode::{Decode, Encode};
use ring::{self, pkcs8, signature};

pub use signature::Ed25519KeyPair as KeyPair;
pub use signature::Signature;

use crate::GenericResult;

pub type PublicKey = <KeyPair as ring::signature::KeyPair>::PublicKey;
/// The length of a public key in bytes.
pub const PUBLIC_KEY_LEN: usize = signature::ED25519_PUBLIC_KEY_LEN;
/// The length of a signature in bytes.
/// TODO(emilio): This is not exposed by ring, seems unfortunate.
pub const SIGNATURE_LEN: usize = 64;

#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub struct MaybeInvalidPublicKey(pub [u8; PUBLIC_KEY_LEN]);

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MaybeInvalidSignature(pub [u8; SIGNATURE_LEN]);

#[derive(Debug)]
pub struct OwnIdentity {
    pub nickname: String,
    pub key_pair: KeyPair,
}

impl OwnIdentity {
    pub fn new(nickname: String, key_pair: KeyPair) -> Self {
        Self { nickname, key_pair }
    }

    pub fn to_public(&self) -> LogicalPeerIdentity {
        use ring::signature::KeyPair;
        LogicalPeerIdentity {
            nickname: self.nickname.clone(),
            key: MaybeInvalidPublicKey(self.key_pair.public_key().as_ref().try_into().unwrap()),
        }
    }
}

fn display_logical_id(nickname: &str, key: &[u8], f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use std::fmt::Write;
    f.write_str(nickname)?;
    f.write_char('#')?;
    // Take the first 3 bytes of the public key to differentiate users with the same nickname.
    for c in key.iter().take(3) {
        write!(f, "{:02x}", *c)?;
    }
    Ok(())
}

impl std::fmt::Display for OwnIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ring::signature::KeyPair;
        display_logical_id(&self.nickname, self.key_pair.public_key().as_ref(), f)
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct LogicalPeerIdentity {
    pub nickname: String,
    pub key: MaybeInvalidPublicKey,
}

impl std::fmt::Display for LogicalPeerIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_logical_id(&self.nickname, &self.key.0, f)
    }
}

/// Generates a key pair from the system rng, returning both the keypair and the pkcs8 document for
/// convenience.
pub fn new_key_pair() -> GenericResult<(KeyPair, pkcs8::Document)> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair = key_pair_from_pkcs8_bytes(pkcs8.as_ref())?;
    Ok((key_pair, pkcs8))
}

/// Creates a key pair from a pkcs8 document.
pub fn key_pair_from_pkcs8_bytes(bytes: &[u8]) -> GenericResult<KeyPair> {
    Ok(signature::Ed25519KeyPair::from_pkcs8(bytes)?)
}

/// Creates a random key pair for signing, and wraps it in an own id.
/// TODO(emilio): This is mostly for convenience for now, consider removing once persistence is
/// implemented and so on.
pub fn new_own_id(nickname: String) -> GenericResult<OwnIdentity> {
    let (kp, _) = new_key_pair()?;
    Ok(OwnIdentity::new(nickname, kp))
}

pub fn sign(key: &KeyPair, msg: &[u8]) -> signature::Signature {
    let sig = key.sign(msg);
    debug_assert_eq!(
        sig.as_ref().len(),
        SIGNATURE_LEN,
        "Unexpected signature length!"
    );
    sig
}

pub fn verify(
    key: &MaybeInvalidPublicKey,
    signature: &MaybeInvalidSignature,
    message: &[u8],
) -> GenericResult<()> {
    signature::UnparsedPublicKey::new(&signature::ED25519, &key.0).verify(message, &signature.0)?;
    Ok(())
}
