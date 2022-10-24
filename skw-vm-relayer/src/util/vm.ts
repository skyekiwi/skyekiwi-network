// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import superagent from 'superagent';
import { u8aToHex } from '@skyekiwi/util';
import relayConfig from '../config';
import { hexToU8a } from '@polkadot/util';

export const initEnclave = async (stateFile: string, stateRoot: Uint8Array) => {
  const sr = u8aToHex(stateRoot);
  await superagent
    .post(relayConfig.enclaveRunnerEndpoint + "/init")
    .type('form')
    .send({
      state_file_path: stateFile, 
      state_root: sr
    });
  // console.log(res)
}

export const callEnclave = async  (payload: Uint8Array, stateRoot: Uint8Array) => {
  const res = await superagent
    .post(relayConfig.enclaveRunnerEndpoint + "/call")
    .type('form')
    .send({
      payload: u8aToHex( payload ),
      state_root: u8aToHex( stateRoot )
    });
  
  return hexToU8a( res.text );
}