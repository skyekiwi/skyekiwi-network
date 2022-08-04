use std::path::Path;

use crate::crypto::{KeyType, PublicKey, SecretKey, Signature};

use serde::{Deserialize, Serialize};

/// Generic signer trait, that can sign with some subset of supported curves.
pub trait Signer: Sync + Send {
    fn public_key(&self) -> PublicKey;
    fn sign(&self, data: &[u8]) -> Signature;

    fn verify(&self, data: &[u8], signature: &Signature) -> bool {
        signature.verify(data, &self.public_key())
    }

    /// Used by test infrastructure, only implement if make sense for testing otherwise raise `unimplemented`.
    fn write_to_file(&self, _path: &Path) -> std::io::Result<()> {
        unimplemented!();
    }
}

// Signer that returns empty signature. Used for transaction testing.
pub struct EmptySigner {}

impl Signer for EmptySigner {
    fn public_key(&self) -> PublicKey {
        PublicKey::empty(KeyType::SR25519)
    }

    fn sign(&self, _data: &[u8]) -> Signature {
        Signature::empty(KeyType::SR25519)
    }
}

/// Signer that keeps secret key in memory.
#[derive(Clone, Serialize, Deserialize)]
pub struct InMemorySigner {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl InMemorySigner {
    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        let secret_key = SecretKey::from_seed(key_type, seed);
        Self { public_key: secret_key.public_key(), secret_key }
    }

    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        Self { public_key: secret_key.public_key(), secret_key }
    }
}

impl Signer for InMemorySigner {
    fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    fn sign(&self, data: &[u8]) -> Signature {
        self.secret_key.sign(data)
    }
}
