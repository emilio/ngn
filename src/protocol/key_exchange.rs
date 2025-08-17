//! Key exchange using ECDH.

use crate::protocol::encryption::Keys;
use crate::GenericResult;
use bincode::{Decode, Encode};
pub use ring::agreement::EphemeralPrivateKey as PrivateKey;
use ring::agreement::X25519;
pub use ring::agreement::{PublicKey, UnparsedPublicKey};
use ring::error::Unspecified;
use std::sync::Arc;

use self::KeyExchangeState as State;
use crate::trivial_error;

pub const PUBLIC_KEY_LEN: usize = 32;

#[derive(Debug, Clone, Encode, Decode)]
pub struct MaybeInvalidPublicKey([u8; PUBLIC_KEY_LEN]);

#[derive(Debug)]
enum KeyExchangeState {
    InProgress(PrivateKey),
    Completed(Arc<super::encryption::Keys>),
    Errored,
}

#[derive(Debug)]
pub struct KeyExchange {
    public_key: PublicKey,
    state: State,
}

impl KeyExchange {
    pub fn new() -> Result<Self, Unspecified> {
        let private = ring::agreement::EphemeralPrivateKey::generate(
            &X25519,
            &ring::rand::SystemRandom::new(),
        )?;
        let public_key = private.compute_public_key()?;
        Ok(Self {
            public_key,
            state: State::InProgress(private),
        })
    }

    pub fn export_public_key(&self) -> MaybeInvalidPublicKey {
        MaybeInvalidPublicKey(self.public_key.as_ref().try_into().unwrap())
    }

    pub fn finish(&mut self, peer_key: &MaybeInvalidPublicKey) -> GenericResult<()> {
        if !matches!(self.state, KeyExchangeState::InProgress(..)) {
            return Err(trivial_error!("Exchange already completed"));
        }
        let result = std::mem::replace(&mut self.state, State::Errored);
        self.state = match result {
            KeyExchangeState::InProgress(private) => {
                let peer_key = UnparsedPublicKey::new(&X25519, &peer_key.0[..]);
                State::Completed(Arc::new(Keys::from_shared_secret(private, peer_key)?))
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    /// Returns the encryption keys for this exchange, if the exchange has finished.
    pub fn encryption_keys(&self) -> Option<&Arc<super::encryption::Keys>> {
        match self.state {
            State::Completed(ref k) => Some(k),
            State::Errored | State::InProgress(..) => None,
        }
    }
}
