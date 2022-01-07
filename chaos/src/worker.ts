// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { expose } from 'threads/worker';
import { getLogger, u8aToHex } from '@skyekiwi/util'
import { randomBytes } from 'tweetnacl';
import { sendTx } from './util'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'

const party = {
  async pushCall(index) {
    await waitReady();
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: provider });
    
    const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri("//Alice");
    const keyring = new Keyring({ type: 'sr25519' }).addFromUri(`//${index}`)

    await sendTx(api.tx.balances.transfer(keyring.address, 155_000_142), rootKeypair, getLogger(`fund account //${index}`));

    const logger = getLogger(keyring.address);

    const bytes = randomBytes(32);
    const pushCall = api.tx.sContract.pushCall(
      0, u8aToHex(bytes)
    );

    logger.info(`send tx from ${keyring.address}`)
    await sendTx(pushCall, keyring, logger);
  }
}

// 125_000_142

expose(party)
