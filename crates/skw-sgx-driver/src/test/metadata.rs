// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::metadata::*;
use crate::types::metadata::*;
use sgx_tstd::{convert::TryInto, vec::Vec};

const AUTHOR_PRIVATE: &str = "1234567890123456789012345678904512345678901234567890123456789045";
const RECP_PRIVATE: &str = "1234567890123456789012345678904612345678901234567890123456789046";

const ENCODED_PRE_SEAL: &str = "516d5a4d7051384b375470315577616538535869335a4a714a444553384a4742694d6d4e575632695261747762570000000000000000000000000000000000000000000000000000000000000000123456789012345678901234567890121234567890123456789012345678904500000101";
const CHUNK_CID: &str = "QmZMpQ8K7Tp1Uwae8SXi3ZJqJDES8JGBiMmNWV2iRatwbW";
const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const SLK: &str = "1234567890123456789012345678901212345678901234567890123456789045";
const VERSION: &str = "00000101";

pub fn encode_decode_pre_seal() {  
  let pre_seal = skw_utils::decode_hex(ENCODED_PRE_SEAL).unwrap();
  let chunk_cid = CHUNK_CID.as_bytes();
  let hash = skw_utils::decode_hex(HASH).unwrap();
  let slk = skw_utils::decode_hex(SLK).unwrap();
  let version = skw_utils::decode_hex(VERSION).unwrap();

  let encoded: Vec<u8> = encode_pre_seal(PreSeal {
    chunk_cid: chunk_cid.try_into().unwrap(),
    hash: hash.try_into().unwrap(),
    sealing_key: slk.try_into().unwrap(),
    version: version.try_into().unwrap()
  }).unwrap();

  assert_eq!(encoded, pre_seal);
}

pub fn encode_decode_sealed_one() {
  let pre_seal = skw_utils::decode_hex(ENCODED_PRE_SEAL).unwrap();
  let author_private = skw_utils::decode_hex(AUTHOR_PRIVATE).unwrap();
  let keypair = skw_crypto::nacl::NaClBox::keypair_from_seed(author_private.try_into().unwrap());
  let version = skw_utils::decode_hex(VERSION).unwrap();

  let cipher = skw_crypto::nacl::NaClBox::encrypt(&pre_seal, keypair.public_key).unwrap();
  let cipher_encoded = encode_box_cipher(cipher);

  let original = SealedMetadata {
    sealed: Sealed {
      is_public: false,
      cipher: cipher_encoded,
      members_count: 1
    },
    version: version[..].try_into().expect("version code with incorrect length"),
  };

  let encoded: Vec<u8> = encode_sealed_metadata(original.clone()).unwrap();
  let recovered = decode_sealed_metadata(encoded).unwrap();
  let recoverd_preseal = decrypt_recovered_cipher(&[keypair], recovered.sealed.cipher.clone()).unwrap();

  assert_eq!(recovered, original);
  assert_eq!(recoverd_preseal, pre_seal);
}

pub fn encode_decode_sealed_multiple() {
  let pre_seal = skw_utils::decode_hex(ENCODED_PRE_SEAL).unwrap();
  let author_private = skw_utils::decode_hex(AUTHOR_PRIVATE).unwrap();
  let recp_private = skw_utils::decode_hex(RECP_PRIVATE).unwrap();

  let keypair = skw_crypto::nacl::NaClBox::keypair_from_seed(author_private.try_into().unwrap());
  let keypair2 = skw_crypto::nacl::NaClBox::keypair_from_seed(recp_private.try_into().unwrap());

  let version = skw_utils::decode_hex(VERSION).unwrap();

  let cipher = skw_crypto::nacl::NaClBox::encrypt(&pre_seal, keypair.public_key).unwrap();
  let cipher_encoded = encode_box_cipher(cipher);

  let cipher2 = skw_crypto::nacl::NaClBox::encrypt(&pre_seal, keypair2.public_key).unwrap();
  let cipher2_encoded = encode_box_cipher(cipher2);
  
  let original = SealedMetadata {
    sealed: Sealed {
      is_public: false,
      cipher: [&cipher_encoded[..], &cipher2_encoded[..]].concat(),
      members_count: 2
    },
    version: version[..].try_into().expect("version code with incorrect length"),
  };

  let encoded: Vec<u8> = encode_sealed_metadata(original.clone()).unwrap();
  let recovered = decode_sealed_metadata(encoded).unwrap();

  let recoverd_preseal = decrypt_recovered_cipher(&[keypair], recovered.sealed.cipher.clone()).unwrap();
  let recoverd_preseal_2 = decrypt_recovered_cipher(&[keypair2], recovered.sealed.cipher.clone()).unwrap();

  assert_eq!(recovered, original);
  assert_eq!(recoverd_preseal, pre_seal);
  assert_eq!(recoverd_preseal_2, pre_seal);
}
