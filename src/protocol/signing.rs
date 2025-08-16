//! Signing and encryption of messages.

use crate::GenericResult;
use bincode::{Decode, Encode};
use ring::{self, pkcs8, signature};

pub use signature::Ed25519KeyPair as KeyPair;
pub use signature::Signature;

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

pub fn sign(key: &KeyPair, msg: &[u8]) -> Signature {
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

/// Generates a key pair from the system rng, returning both the keypair and the pkcs8 document for
/// convenience.
pub fn new_key_pair() -> GenericResult<(KeyPair, pkcs8::Document)> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = KeyPair::generate_pkcs8(&rng)?;
    let key_pair = key_pair_from_pkcs8_bytes(pkcs8.as_ref())?;
    Ok((key_pair, pkcs8))
}

/// Creates a key pair from a pkcs8 document.
pub fn key_pair_from_pkcs8_bytes(bytes: &[u8]) -> GenericResult<KeyPair> {
    Ok(KeyPair::from_pkcs8(bytes)?)
}
