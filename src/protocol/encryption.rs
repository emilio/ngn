use ring::aead::{Aad, Nonce, UnboundKey, NONCE_LEN};
use ring::aead::{BoundKey, AES_256_GCM};
use ring::error::Unspecified;

use crate::protocol::key_exchange;

// Seems most AEAD algorithms fail in presence of repeated nonces, so probably
// worth combining randomness with a counter or so:
//
//  * https://docs.rs/ring/0.17.14/src/ring/aead.rs.html#43-47
//  * https://csrc.nist.gov/publications/detail/sp/800-38d/final
//  * https://docs.rs/aead/0.5.2/src/aead/lib.rs.html#151
//  * https://docs.rs/aead/0.5.2/src/aead/stream.rs.html#437
pub struct NonceSequence {
    counter: u32,
    rand: ring::rand::SystemRandom,
}

impl Default for NonceSequence {
    fn default() -> Self {
        Self {
            counter: 0,
            rand: ring::rand::SystemRandom::new(),
        }
    }
}

impl ring::aead::NonceSequence for NonceSequence {
    // called once for each seal operation
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        use ring::rand::SecureRandom;

        let mut nonce_bytes = [0u8; NONCE_LEN];

        let bytes = self.counter.to_be_bytes();
        nonce_bytes[0..4].copy_from_slice(&bytes);
        self.rand.fill(&mut nonce_bytes[4..])?;

        self.counter += 1;
        Ok(Nonce::assume_unique_for_key(nonce_bytes))
    }
}

const AES_256_KEY_LEN: usize = 256 / 8;
pub type SealingKey = ring::aead::SealingKey<NonceSequence>;
pub type OpeningKey = ring::aead::OpeningKey<NonceSequence>;

#[derive(Debug)]
pub struct Keys {
    pub encryption: SealingKey,
    pub decryption: OpeningKey,
}

impl Keys {
    pub fn from_shared_secret(
        exchange_private_key: key_exchange::PrivateKey,
        peer_public_key: key_exchange::UnparsedPublicKey<&[u8]>,
    ) -> Result<Self, Unspecified> {
        let key_bytes: [u8; AES_256_KEY_LEN] = ring::agreement::agree_ephemeral(
            exchange_private_key,
            &peer_public_key,
            |shared_secret: &[u8]| shared_secret.try_into(),
        )??;
        Ok(Self {
            encryption: SealingKey::new(
                UnboundKey::new(&AES_256_GCM, &key_bytes).unwrap(),
                NonceSequence::default(),
            ),
            decryption: OpeningKey::new(
                UnboundKey::new(&AES_256_GCM, &key_bytes).unwrap(),
                NonceSequence::default(),
            ),
        })
    }

    pub fn encrypt_in_place_append_tag(&mut self, data: &mut Vec<u8>) -> Result<(), Unspecified> {
        self.encryption
            .seal_in_place_append_tag(Aad::from(b""), data)
    }

    pub fn decrypt_in_place<'a>(
        &mut self,
        data: &'a mut [u8],
    ) -> Result<&'a mut [u8], Unspecified> {
        self.decryption.open_in_place(Aad::from(b""), data)
    }
}
