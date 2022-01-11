// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { randomBytes } from 'tweetnacl';
import { getLogger, u8aToHex } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import {sendTx} from './util'

export class Chaos {

  public async letsParty(accountIndex: number, loop: number) {
    await waitReady();
    
    const keyring = new Keyring({ type: 'sr25519' }).addFromUri(`//${accountIndex}`)
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: provider });

    // deploy a shit tons of contract -> contractId cannot be wrongfully read
    for (let i = 0 ; i < loop; i ++) {
      const logger = getLogger(`push calls to //${accountIndex}`);
      const bytes = randomBytes(32);
      const pushCall = api.tx.sContract.pushCall(0, u8aToHex(bytes));
      logger.info(`pushing calls from ${keyring.address}`)
      await sendTx(pushCall, keyring, logger);  
    }
  }
}
