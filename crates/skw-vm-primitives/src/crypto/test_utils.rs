use rand::rngs::StdRng;

use crate::crypto::signature::{
    ED25519PublicKey, ED25519SecretKey, KeyType, PublicKey, SecretKey, SECP256K1,
};
use crate::crypto::{InMemorySigner, Signature};

fn ed25519_key_pair_from_seed(seed: &str) -> ed25519_dalek::Keypair {
    let seed_bytes = seed.as_bytes();
    let len = std::cmp::min(ed25519_dalek::SECRET_KEY_LENGTH, seed_bytes.len());
    let mut seed: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = [b' '; ed25519_dalek::SECRET_KEY_LENGTH];
    seed[..len].copy_from_slice(&seed_bytes[..len]);
    let secret = ed25519_dalek::SecretKey::from_bytes(&seed).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    ed25519_dalek::Keypair { secret, public }
}

fn sr25519_secret_key_from_seed(seed: &str) -> schnorrkel::MiniSecretKey {
    let seed_bytes = seed.as_bytes();
    let len = std::cmp::min(schnorrkel::MINI_SECRET_KEY_LENGTH, seed_bytes.len());
    let mut seed: [u8; schnorrkel::MINI_SECRET_KEY_LENGTH] = [b' '; schnorrkel::MINI_SECRET_KEY_LENGTH];
    seed[..len].copy_from_slice(&seed_bytes[..len]);
    let secret = schnorrkel::MiniSecretKey::from_bytes(&seed).unwrap();
    secret
}

fn secp256k1_secret_key_from_seed(seed: &str) -> secp256k1::key::SecretKey {
    let seed_bytes = seed.as_bytes();
    let len = std::cmp::min(32, seed_bytes.len());
    let mut seed: [u8; 32] = [b' '; 32];
    seed[..len].copy_from_slice(&seed_bytes[..len]);
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
    secp256k1::key::SecretKey::new(&SECP256K1, &mut rng)
}

impl PublicKey {
    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        match key_type {
            KeyType::ED25519 => {
                let keypair = ed25519_key_pair_from_seed(seed);
                PublicKey::ED25519(ED25519PublicKey(keypair.public.to_bytes()))
            },
            KeyType::SR25519 => {
                let secret = sr25519_secret_key_from_seed(seed);
                PublicKey::ED25519(ED25519PublicKey(
                    secret.expand_to_public(schnorrkel::ExpansionMode::Ed25519).to_bytes()
                ))
            },
            _ => unimplemented!(),
        }
    }
}

impl SecretKey {
    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        match key_type {
            KeyType::ED25519 => {
                let keypair = ed25519_key_pair_from_seed(seed);
                SecretKey::ED25519(ED25519SecretKey(keypair.to_bytes()))
            },
            KeyType::SR25519 => {
                let key = sr25519_secret_key_from_seed(seed);
                SecretKey::SR25519(key)
            },
            _ => SecretKey::SECP256K1(secp256k1_secret_key_from_seed(seed)),
        }
    }
}

impl Signature {
    /// Empty signature that doesn't correspond to anything.
    pub fn empty(key_type: KeyType) -> Self {
        match key_type {
            KeyType::ED25519 => {
                Signature::ED25519(ed25519_dalek::Signature::from_bytes(&[0u8; ed25519_dalek::SIGNATURE_LENGTH]).unwrap())
            },
            KeyType::SR25519 => {
                Signature::SR25519(schnorrkel::Signature::from_bytes(&[
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8,// Gotta mark the sig as Schnorrkel
                ]).unwrap())
            },
            _ => unimplemented!(),
        }
    }
}

impl InMemorySigner {
    pub fn from_random(key_type: KeyType) -> Self {
        let secret_key = SecretKey::from_random(key_type);
        Self { public_key: secret_key.public_key(), secret_key }
    }
}
