// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path'

import { Driver } from '@skyekiwi/driver';
import { AsymmetricEncryption, DefaultSealer, EncryptionSchema } from '@skyekiwi/crypto';

import fs from 'fs'
import {IPFS} from '@skyekiwi/ipfs'

import { getLogger, stringToU8a } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { sendTx } from './util'
import { File } from '@skyekiwi/file';
import { u8aToHex } from '@skyekiwi/util'
import { Calls, Call, buildCalls } from '@skyekiwi/s-contract';
import {baseDecode} from 'borsh';
import { encode } from 'punycode';

require("dotenv").config()

const genesis = async () => {

  const logger = getLogger("genesis");

  await waitReady();
  const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri('//Alice');

  const provider = new WsProvider('ws://127.0.0.1:9944');
  // const provider = new WsProvider('wss://staging.rpc.skye.kiwi');
  const api = await ApiPromise.create({ provider: provider });

  const shardKey = new Uint8Array([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    0, 1
  ]);
  const publicKey = AsymmetricEncryption.getPublicKey(shardKey);

  // 1. register and initialize a shard 
  const registerSecretKeeper = api.tx.registry.registerSecretKeeper(
    '0x' + u8aToHex(publicKey) ,"00000000"
  )
  const registerShard = api.tx.registry.registerRunningShard(0);
   const authorizeRoot = api.tx.sudo.sudo(
      api.tx.sContract.addAuthorizedShardOperator(0, rootKeypair.address)
  );


  let shardInitializeCalls = new Calls({
    ops: [
      new Call({
        origin_public_key: publicKey,
        receipt_public_key: publicKey,
        encrypted_egress: false,
  
        transaction_action: 0,
        amount: 1000000,
        wasm_blob_path: null,
        method: null,
        args: null
      })
    ],
    block_number: null,
    shard_id: 0,
  })

  const fundAccounts: any[] = []
  // 3. fund the accounts from the root account
  for (let i = 1; i <= 10; i++) {
      const keyring = (new Keyring({ type: 'sr25519' })).addFromUri(`//${i}`);
      // fund the account with enough gas for 20 push calls
      fundAccounts.push(api.tx.balances.transfer(keyring.address, 155_000_142 * 1000));

      shardInitializeCalls.ops.push(new Call({
        origin_public_key: publicKey,
        receipt_public_key: publicKey,
        encrypted_egress: false,
  
        transaction_action: 0,
        amount: 10,
        wasm_blob_path: null,
        method: null,
        args: null,
      }));
  }

  shardInitializeCalls.ops.push(new Call({
    origin_public_key: publicKey,
    receipt_public_key: publicKey,
    encrypted_egress: false,

    transaction_action: 4,
    amount: 1,
    wasm_blob_path: stringToU8a('/Users/songzhou/Desktop/skyekiwi-network/mock-enclave/wasm/status_message_collections.wasm'),
    method: null,
    args: null,
  }));

  fundAccounts.push(api.tx.balances.transfer(  "5DFhSMLmnw3Fgc6trbp8AuErcZoJS64gDFHUemqh2FRYdtoC"  , 155_000_142 * 20));

  const encryptionSchema = new EncryptionSchema();
  encryptionSchema.addMember(publicKey);

  let initializeShard
  await Driver.upstream(
      new File({
          fileName: "empty_state",
          readStream: fs.createReadStream(path.join(__dirname, '../mock/empty__state_dump__ColState'))
      }),
      new DefaultSealer(), encryptionSchema,
      async (cid: string) => {

          const encodedCall = u8aToHex(new Uint8Array(baseDecode( buildCalls(shardInitializeCalls) )));
          initializeShard = api.tx.sContract.initializeShard(
              0, '0x' + encodedCall, cid,
              publicKey
          );
      }
  )

  const shardConfirmationThreshold = api.tx.sudo.sudo(
    api.tx.parentchain.setShardConfirmationThreshold(0, 1)
  );

  const wasmBlobSM = new Uint8Array(fs.readFileSync(path.join(__dirname, '../wasm/status_message_collections.wasm')));
  // const wasmBlobFT = new Uint8Array(fs.readFileSync(path.join(__dirname, '../wasm/fungible_token.wasm')));

  const cidSM = await IPFS.add(u8aToHex(wasmBlobSM));
  // const cidFT = await IPFS.add(u8aToHex(wasmBlobFT));
  const deploymentCalls = new Calls({ ops: [ ], block_number: null, shard_id: 0 });
  const encodedDeploymentCall = '0x' + u8aToHex(new Uint8Array(baseDecode( buildCalls(deploymentCalls) ))) 
  const deployContract = [
    api.tx.sContract.registerContract(
      "status_message_collections", cidSM.cid.toString(), encodedDeploymentCall,  0
    ),
    // api.tx.sContract.registerContract(
    //   "skw_token", cidFT.cid.toString(), buildCalls(deploymentCalls), 0
    // ),
    // api.tx.sContract.registerContract(
    //   "dot_token", cidFT.cid.toString(), buildCalls(deploymentCalls), 0
    // ),
    // api.tx.sContract.registerContract(
    //   "usdt_token", cidFT.cid.toString(), buildCalls(deploymentCalls), 0
    // )
  ];

  const submitInitialize = api.tx.utility.batch(
    [
      ...fundAccounts,
      registerSecretKeeper, registerShard,
      authorizeRoot, initializeShard, shardConfirmationThreshold,
      ...deployContract,
    ]
  );
  await sendTx(submitInitialize, rootKeypair, logger);
}

export {genesis}
