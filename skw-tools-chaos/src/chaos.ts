// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { randomBytes } from 'tweetnacl';
import { stringToU8a, u8aToHex, sendTx, sleep } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { Calls, Call, buildCalls } from '@skyekiwi/s-contract';
import {blake2AsU8a} from '@polkadot/util-crypto'

export class Chaos {

  public async letsParty(accountIndex: number, loop: number) {
    await waitReady();

    const keyring = new Keyring({ type: 'sr25519' }).addFromUri(`//${accountIndex}`)
    const provider = new WsProvider('ws://127.0.0.1:8844');
    const api = await ApiPromise.create({ provider: provider });

    await sendTx(
      api.tx.sAccount.createAccount(0), keyring,
    );
    for (let i = 0 ; i < loop; i ++) {

      const call = new Calls({
        ops: [
          new Call({
            origin_public_key: keyring.publicKey,
            receipt_public_key: blake2AsU8a('status_message'),
            encrypted_egress: false,

            transaction_action: 2,
            contract_name: stringToU8a('status_message'),
            amount: null,
            method: stringToU8a('set_status'),
            args: stringToU8a(JSON.stringify({message: "0x" + u8aToHex(randomBytes(32))})),
            wasm_code: null,
          }),
          new Call({
            origin_public_key: keyring.publicKey,
            receipt_public_key: blake2AsU8a('status_message'),
            encrypted_egress: false,

            transaction_action:3,
            contract_name: stringToU8a('status_message'),
            amount: null,
            method: stringToU8a('get_status'),
            args: stringToU8a(JSON.stringify({account_id: keyring.address.toLowerCase()})),
            wasm_code: null,
          })
        ],
        block_number: 0,
        shard_id: 0,
      })

      const pushCall = api.tx.sContract.pushCall(0, '0x' + u8aToHex( new Uint8Array( buildCalls(call))));
      await sendTx(pushCall, keyring);

      const random = Math.floor(Math.random() * (1000 - 1)) + 1;
      await sleep(random * 100);
    }
  }
}
