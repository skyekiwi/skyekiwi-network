// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { randomBytes } from 'tweetnacl';
import { getLogger } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import PQueue from 'p-queue';
import { spawn, Worker } from 'threads'
import {sendTx} from './util'

export class Chaos {
  #queue: PQueue

  constructor() {
    this.#queue = new PQueue({ concurrency: 1 })
  }

  public async letsParty() {
    await waitReady();
    const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri("//Alice");

    let pools: any[] = []

    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: provider });

    for (let i = 0; i < 10; i++) {
      const pool = await spawn(new Worker('./worker'));
      pools.push(pool);
    }

    const registerContract = api.tx.sContract.registerContract(
      'QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N',
      '38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f',
      '1111111111222222222211111111112222222222'
    );

    await sendTx(registerContract, rootKeypair, getLogger('deployContract'));

    for (let i = 0; i < 100; i++) {
      const accountIndex = randomBytes(1)[0] % 10;
      this.#queue.add(async () => 
        await pools[accountIndex].pushCall(accountIndex))
    }
  }
}
