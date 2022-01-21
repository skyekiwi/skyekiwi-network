// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use skw_utils::{encode_hex, decode_hex};
use skw_crypto::*;

const TEST: &str = "010203040a0b";
const ANSWER: &[u8] = &[1, 2, 3, 4, 10, 11];

pub fn encode_decode_hex() {
  // static content
  let answer = decode_hex(TEST);
  assert_eq!(answer.unwrap(), ANSWER);

  let result = encode_hex(ANSWER);
  assert_eq!(result.unwrap(), TEST);

  // random bytes
  let bytes = random_bytes!(40);
  let result = encode_hex(&bytes).unwrap();
  let re_decoded = decode_hex(&result[..]).unwrap();
  assert_eq!(re_decoded, bytes);
}
