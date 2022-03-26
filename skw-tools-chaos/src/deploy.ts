// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path'

import {IPFS} from '@skyekiwi/ipfs'
import fs from 'fs'
import { getLogger } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { sendTx } from './util'
import { u8aToHex } from '@skyekiwi/util'
import { Calls, buildCalls } from '@skyekiwi/s-contract';

require("dotenv").config()

const deployContract = async () => {

  const logger = getLogger("genesis");

  await waitReady();
  const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri("//Alice");

  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider: provider });

  const wasmBlob = new Uint8Array(fs.readFileSync(path.join(__dirname, '../wasm/status_message_collections.wasm')));

  const ipfs = new IPFS();
  const cid = await ipfs.add(u8aToHex(wasmBlob));
  const deploymentCalls = new Calls({ ops: [ ] });
  const deployContract = api.tx.sContract.registerContract(
    "status_message_collections", cid.cid.toString(), buildCalls(deploymentCalls), 0
  );

  await sendTx(deployContract, rootKeypair, logger);

  await provider.disconnect();
}

export {deployContract}
