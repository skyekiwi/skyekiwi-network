// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import Level from 'level';
import fs from 'fs';

import { execSync } from 'child_process';
import { IPFS } from '@skyekiwi/ipfs'
import {  baseDecode, baseEncode, BlockSummary, buildCalls, Calls } from '@skyekiwi/s-contract';
import { ExecutionSummary, Outcomes, parseRawOutcomes } from '@skyekiwi/s-contract/borsh';
import { getLogger, u8aToHex, padSize, unpadSize } from '@skyekiwi/util';
import { hexToU8a, u8aToString } from '@polkadot/util';
import { encodeAddress } from '@polkadot/util-crypto';
import { ApiPromise } from '@polkadot/api';

import { Storage } from './storage';
import bridgeConfig from '../config';
import { DBOps } from './types';

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Dispatcher {

  public static async isDispatchable(db: Level.LevelDB, blockNumber: number): Promise<boolean> {
    const summary = await Storage.getExecutionSummary(db);
    return summary.high_local_execution_block < blockNumber;
  }

  public static async preDispatchProcessing(
    api: ApiPromise, db: Level.LevelDB, callsIndex: number
  ): Promise<Calls> {

    const logger = getLogger('dispatcher.preDispatchProcessing');

    const c = await Storage.getCallsRecord(db, 0, callsIndex)

    const blockNumber = c.block_number;
    const shardId = c.shard_id;

    let res = new Calls({
      ops: [],
      shard_id: shardId,
      block_number: blockNumber,
    });

    for (let op of c.ops) {
      const originAddress = encodeAddress(op.origin_public_key);
      switch(op.transaction_action) {
        
        
        // 0. if its an account creation call
        case 0:  
          // 0.1 Verification
          
          if (originAddress !== '5EYCAe5jKbSe4DzkVVriG3QW13WG4j9gy4zmUxjqT8czBuyu') {
            throw new Error("Wrong Origin - this should never happen");
          }

          // 0.3 Put the call to res
          res.ops.push(op);
        break;

        // 1. if its a transfer action
        case 1:
          // NOT ALLOWED!
        break;

        // 3. if its a function call
        case 2:
          // Do nothing
          res.ops.push(op);
        break;

        // 4. if its a view method call
        case 3:
          res.ops.push(op);
        break;

        // 4. if its a smart contract deployment call
        case 4:
          // 0.1 Verification
          if (originAddress !== '5EYCAe5jKbSe4DzkVVriG3QW13WG4j9gy4zmUxjqT8czBuyu') {
            throw new Error("Wrong Origin - this should never happen");
          }

          // 0.2 Fetch contract 
          const contractName = u8aToString(op.contract_name);
          const wasmBlobCID = await api.query.sContract.wasmBlobCID(shardId, contractName);
          const content = await IPFS.cat(u8aToString(hexToU8a(wasmBlobCID.toString().substring(2))) );
          const wasmPath = `${bridgeConfig.localWASMStorage}/${u8aToHex(op.receipt_public_key)}.wasm`;
          fs.writeFileSync(wasmPath, hexToU8a(content));

          // 0.3 Put the call to res
          res.ops.push(op);
        break;
      }
    }

    logger.info(`👀 call validated for ${callsIndex}, sending to executor`)
    return res;
  }

  public static callRuntime(encodedBlock: string, stateRoot: Uint8Array): string {  
    return JSON.parse(execSync(`${bridgeConfig.vmBinaryPath} \
      --state-file ${bridgeConfig.stateDumpPrefix} \
      --state-root ${u8aToHex(stateRoot)} \
      --params ${encodedBlock} \
      --wasm-files-base ${bridgeConfig.localWASMStorage} \
      --dump-state`
    ).toString())
  }

  public static _recoverState(originFile: string, outputFile: string, rawPatches: Uint8Array[]): void {
    let patch = new Uint8Array(0);
    for (const rawPatch of rawPatches) {
      patch = new Uint8Array([...patch, ...padSize(rawPatch.length), ...rawPatch]);
    }

    execSync(`${bridgeConfig.statePatcherBinaryPath} \
      --state-file ${originFile} \
      --state-patch ${u8aToHex(patch)} \
      --output ${outputFile}`
    )
  }

  public static  dispatchBatch(
    calls: {[key: number]: Calls},
    lastStateRoot: Uint8Array
  ): [DBOps[], Uint8Array] {
    const logger = getLogger('dispatcher.dispatchBatch');

    if (Object.keys(calls).length === 0) {
      return [[], new Uint8Array(0)];
    }

    // TODO: maybe verify all calls are in the same block/shard?
    const blockNumber = calls[Object.keys(calls)[0] as unknown as number].block_number;

    logger.info(`🛠 dispatching block_number #${blockNumber}`);

    // 0. setups
    let dbOps: DBOps[] = [];

    let blockSummary = new BlockSummary({
      block_number: blockNumber,
      block_state_root: new Uint8Array(0),
      contract_state_patch_from_previous_block: new Uint8Array(0),
      call_state_patch_from_previous_block: new Uint8Array(0),
    })

    // 1. Pack a block of calls
    /*
      callRaw:
      BlockNumber #X [
        [SIZE, CALL_INDEX, ENCODED_CALL]
        [SIZE, CALL_INDEX, ENCODED_CALL]
        [SIZE, CALL_INDEX, ENCODED_CALL]
        ...
      ]
    */
    let callRaw = new Uint8Array(0);
    for (const callIndex in calls) {
      const call = calls[callIndex];
      const encodedCall = baseDecode(buildCalls(call));
      callRaw = new Uint8Array([
        ...callRaw,
        ...padSize(encodedCall.length + 4),
        ...padSize(Number(callIndex)),
        ...encodedCall,
      ]);
    }

    // 2. Send For execution
    const allCallOutcomes = this.callRuntime(
      baseEncode(callRaw), lastStateRoot,
    );

    // 3. Parse the outcomes
    const callsOutcomes = new Uint8Array(baseDecode(allCallOutcomes));
    let callOutcomeOffset = 0;
    let latestStateRoot = new Uint8Array(0);

    /*
    callOutcome:
    BlockNumber #X [
      [SIZE, CALL_INDEX, ENCODED_OUTCOME]
      [SIZE, CALL_INDEX, ENCODED_OUTCOME]
      [SIZE, CALL_INDEX, ENCODED_OUTCOME]
      ...
    ]
  */
    while (callOutcomeOffset < callsOutcomes.length) {
      const outcomeSize = unpadSize(callsOutcomes.slice(callOutcomeOffset, callOutcomeOffset + 4));

      if (outcomeSize === 0) {
        // when would this happen? Empty blocks won't be sent for execution
        break;
      }

      const callIndex = unpadSize(callsOutcomes.slice(callOutcomeOffset + 4, callOutcomeOffset + 8));
      const rawOutcome = parseRawOutcomes(
        baseEncode(
          callsOutcomes.slice(callOutcomeOffset + 8, callOutcomeOffset + 4 + outcomeSize)
        )
      );
  
      callOutcomeOffset += 4 + outcomeSize;
      blockSummary.block_state_root = rawOutcome.state_root;
      blockSummary.call_state_patch_from_previous_block = rawOutcome.state_patch;
      latestStateRoot = rawOutcome.state_root;
 
      const outcome = new Outcomes({
        ops: rawOutcome.ops,
        state_root: latestStateRoot,
      });

      dbOps.push(Storage.writeCallOutcome(0, callIndex, outcome));
    }
  
    dbOps.push(Storage.writeBlockSummary(0, blockNumber, blockSummary));
    dbOps.push(Storage.writeExecutionSummary(new ExecutionSummary({
      high_local_execution_block: blockNumber,
      latest_state_root: latestStateRoot,
    })))
    return [dbOps, latestStateRoot];
  }
}
