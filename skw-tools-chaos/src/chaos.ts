// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { randomBytes } from 'tweetnacl';
import { getLogger, u8aToHex } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import {sendTx} from './util'
import { Calls, Call, buildCalls } from '@skyekiwi/s-contract';


const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export class Chaos {

  public async letsParty(accountIndex: number, loop: number) {
    await waitReady();
    
    const keyring = new Keyring({ type: 'sr25519' }).addFromUri(`//${accountIndex}`)
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: provider });

    for (let i = 0 ; i < loop; i ++) {
      const logger = getLogger(`push calls to //${accountIndex}`);

      const call = new Calls({
        ops: [
          new Call({
            origin: keyring.address,
            origin_public_key: keyring.publicKey,
            encrypted_egress: false,

            transaction_action: 'call',
            receiver: 'status_message_collections',
            amount: null,
            method: 'set_status',
            args: JSON.stringify({message: "0x" + u8aToHex(randomBytes(32))}),
            wasm_blob_path: null,
            to: null,
          }),
          new Call({
            origin: keyring.address,
            origin_public_key: keyring.publicKey,
            encrypted_egress: false,

            transaction_action: 'view_method_call',
            receiver: 'status_message_collections',
            amount: null,
            method: 'get_status',
            args: JSON.stringify({account_id: keyring.address.toLowerCase()}),
            wasm_blob_path: null,
            to: null,
          })
        ]
      })

      const pushCall = api.tx.sContract.pushCall(0, buildCalls(call));
      logger.info(`pushing calls from ${keyring.address}`)
      await sendTx(pushCall, keyring, logger);

      const random = Math.floor(Math.random() * (1000 - 1)) + 1;
      await sleep(random * 100);
    }
  }
}
