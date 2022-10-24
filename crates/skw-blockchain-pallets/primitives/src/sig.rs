use ed25519_dalek::{Signer, Verifier};

use crate::types::PoASingature;

pub fn sign_ed25519(secret_key: &[u8; 32], msg: &[u8]) -> PoASingature {
    let pk = sk_to_pk_ed25519(&secret_key);

    let kp = [&secret_key[..], &pk[..]].concat();
    let kp = ed25519_dalek::Keypair::from_bytes(&kp[..]).unwrap();

    kp.sign(msg).to_bytes().into()
}

pub fn verify_ed25519(public_key: &[u8; 32], msg: &[u8], sig: &[u8; 64]) -> bool {
    let pk = ed25519_dalek::PublicKey::from_bytes(public_key).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&sig[..]).unwrap();
    pk.verify(msg, &sig).is_ok()
}

pub fn sk_to_pk_ed25519<'a>(secret_key: &'a [u8; 32]) -> [u8; 32] {
    let sk = ed25519_dalek::SecretKey::from_bytes(&secret_key[..]).unwrap();
    let pk: ed25519_dalek::PublicKey = (&sk).into();
    pk.to_bytes()
}

#[test]
fn sign_n_verify() {

    let secret_key = [0u8; 32];
    let pk = sk_to_pk_ed25519(&secret_key);
    let msg = [0u8; 100];

    let sig = sign_ed25519(&secret_key, &msg[..]);
    let v = verify_ed25519(&pk, &msg[..], &sig);

    // println!("{:?} {:?}", sig, v);
    assert!(v == true);
}