// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::file::FileHandle;

pub fn inflate_deflat() {
  
  let source = [
    84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 103, 108,
    111, 98, 97, 108, 32, 114, 117, 115, 116, 32, 83, 116, 
    114, 105, 110, 103, 32, 105, 110, 105, 116, 32, 98, 121, 
    32, 108, 97, 122, 121, 95, 115, 116, 97, 116, 105, 99, 33
  ];

  let deflated = FileHandle::deflate_chunk(source.to_vec());
  let recovered = FileHandle::inflate_chunk(deflated);
	assert_eq!(source.to_vec(), recovered)
}

pub fn inflate_client_side() {
  
  let deflated = [
      120, 156,  11, 201, 200,  44,  86,   0, 162,  68,
      133, 244, 156, 252, 164, 196,  28, 133, 162, 210,
      226,  18, 133, 224, 146, 162, 204, 188, 116, 133,
      204, 188, 204,  18, 133, 164,  74, 133, 156, 196,
      170, 202, 248, 226, 146, 196, 146, 204, 100,  69,
        0, 183, 150,  17, 227
  ];

let source = [
    84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 103, 108,
    111, 98, 97, 108, 32, 114, 117, 115, 116, 32, 83, 116, 
    114, 105, 110, 103, 32, 105, 110, 105, 116, 32, 98, 121, 
    32, 108, 97, 122, 121, 95, 115, 116, 97, 116, 105, 99, 33
  ];

  let recovered = FileHandle::inflate_chunk(deflated.to_vec());
	assert_eq!(source.to_vec(), recovered)
}


pub fn sha256_checksum() {
  
  let source = [
    84, 104, 105, 115, 32, 105, 115, 32, 97, 
    32, 103, 108, 111, 98, 97, 108, 32, 114, 
    117, 115, 116, 32, 83, 116, 114, 105, 110, 
    103, 32, 105, 110, 105, 116, 32, 98, 121, 
    32, 108, 97, 122, 121, 95, 115, 116, 97, 
    116, 105, 99, 33
  ];

  let result = [
    169, 234,  74, 197, 148, 176, 200,
      94, 241, 173, 164, 173,   8, 240,
    249,   3, 191, 104,  87, 236, 224,
    244, 126,  10, 115, 194, 105, 146,
    209,  44, 216,  83
  ];

  let hash = FileHandle::sha256_checksum(source.to_vec());
	assert_eq!(hash, result)
}

