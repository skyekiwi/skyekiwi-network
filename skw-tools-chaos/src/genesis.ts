// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path'
import fs from 'fs'

import { Driver } from '@skyekiwi/driver';
import { sendTx , u8aToHex} from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { Calls, buildCalls } from '@skyekiwi/s-contract';
import { AsymmetricEncryption, initWASMInterface, secureGenerateRandomKey } from '@skyekiwi/crypto';
import { KeypairType } from '@skyekiwi/crypto/types';

require("dotenv").config()

const genesis = async () => {
  await initWASMInterface();
  const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri('//Alice');

  const provider = new WsProvider('ws://127.0.0.1:9944');
  // const provider = new WsProvider('wss://staging.rpc.skye.kiwi');
  const api = await ApiPromise.create({ provider: provider });

  const sk = {
    key: secureGenerateRandomKey(),
    keyType: 'sr25519' as KeypairType
  };

  const pk = {
    key: AsymmetricEncryption.getPublicKeyWithCurveType('sr25519', sk.key),
    keyType: 'sr25519' as KeypairType
  };

  // 1. register and initialize a shard 
   const authorizeRoot = api.tx.sudo.sudo(
      api.tx.sContract.addAuthorizedShardOperator(0, rootKeypair.address)
  );

  const fundAccounts: any[] = []

  // 3. fund the accounts from the root account
  for (let i = 1; i <= 15; i++) {
      const keyring = (new Keyring({ type: 'sr25519' })).addFromUri(`//${i}`);
      // fund the account with enough gas for 20 push calls
      fundAccounts.push(api.tx.balances.transfer(keyring.address, 10 * (10 ** 12)));
  }
  fundAccounts.push(api.tx.balances.transfer(  "5DFhSMLmnw3Fgc6trbp8AuErcZoJS64gDFHUemqh2FRYdtoC"  , 155_000_142 * 20));

  const file = fs.readFileSync(path.join(__dirname, '../mock/empty__state_dump__ColState'));

  const preSealed = await Driver.generatePreSealedData(new Uint8Array(file));
  const sealed = Driver.generateSealedData(preSealed, [pk], false);

  const initShard = api.tx.sContract.initializeShard(
    0, sealed.serialize(), pk.key
  );

  const shardConfirmationThreshold = api.tx.sudo.sudo(
    api.tx.parentchain.setShardConfirmationThreshold(0, 1)
  );

  const wasmBlobSM = new Uint8Array(fs.readFileSync(path.join(__dirname, '../wasm/status_message.wasm')));
  // // const wasmBlobFT = new Uint8Array(fs.readFileSync(path.join(__dirname, '../wasm/fungible_token.wasm')));
  
  const deploymentCalls = new Calls({ ops: [ ], block_number: null, shard_id: 0 });
  const encodedDeploymentCall = '0x' + u8aToHex(new Uint8Array(buildCalls(deploymentCalls)))
  const deployContract = api.tx.sContract.registerContract(
    "status_message", "0x" + u8aToHex(wasmBlobSM), encodedDeploymentCall,  0
  )

  const submitInitialize = api.tx.utility.batch(
    [
      ...fundAccounts,
      authorizeRoot, initShard, shardConfirmationThreshold, deployContract
    ]
  );
  await sendTx(submitInitialize, rootKeypair);
}

export {genesis}
