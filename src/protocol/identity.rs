//! Ed25519 utils, signing and verification.

use super::signing::{KeyPair, MaybeInvalidPublicKey};
use crate::GenericResult;
use bincode::{Decode, Encode};

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

/// Creates a random key pair for signing, and wraps it in an own id.
/// TODO(emilio): This is mostly for convenience for now, consider removing once persistence is
/// implemented and so on.
pub fn new_own_id(nickname: String) -> GenericResult<OwnIdentity> {
    let (kp, _) = super::signing::new_key_pair()?;
    Ok(OwnIdentity::new(nickname, kp))
}
