// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import EventEmitter from "events";

import { Call, parseCalls, parseOutcomes, baseDecode, baseEncode, buildCalls, Calls } from '@skyekiwi/s-contract';
import { hexToU8a, u8aToString, padSize, unpadSize, sleep } from '@skyekiwi/util';
import { decodeAddress, encodeAddress } from '@polkadot/util-crypto';

import { initEnclave, callEnclave, DB } from '../util';
import bridgeConfig from '../config';

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Dispatcher {
  #active: boolean
  #query: string
  #db: DB
  #shards: number[]
  #progress?: EventEmitter

  constructor(db: DB, progress?: EventEmitter) {
    if (!db.isReady()) {
      throw new Error("db not connected");
    }
    this.#query = "";
    this.#active = true;
    this.#db = db;
    this.#shards = []
    this.#progress = progress;
  }

  public async shutdown() {
    await this.writeAll();
    this.#active = false;
  }

  public async writeAll (): Promise<void> {
    if ( this.#active && this.#query ) {
      await this.#db.commitBlockQuery(this.#query);
      this.#query = "";
    }
  }

  public async setShards(shards: number[]) {
    this.#shards = shards;
  }

  public async dispatchAll() {
    while(this.#active) {
      for (const shardId of this.#shards) {
        let latestStateRoot = await this.#db.getLatestStateRoot(shardId);
        const blocks = await this.#db.selectBlocksForExecution(shardId);

        if (!blocks || blocks.length === 0) {
          if (this.#progress) this.#progress.emit("progress", "DISPATCHER_EXECUTION_SKIPPING");
          // Sleep for a block time
          sleep(6000);
        }
        for (const block of blocks) {
          let payload: Uint8Array = new Uint8Array(0);
          for (const call of block.calls) {
            const chainOriginPublicKey = decodeAddress(call.origin);
            try {
              const c = parseCalls( baseEncode( call.encoded ) );
              console.log(c);
              const validatedCalls = new Calls({
                ops: [],
                shard_id: shardId,
                block_number: block.block_number
              });
              for (const op of c.ops) {
                validatedCalls.ops.push(await this.preProcessCall(shardId, op));
              }

              if (this.#progress) this.#progress.emit("progress", "DISPATCHER_EXECUTION_CALL_VALIDATED", call.call_index);

              if (this.#progress) this.#progress.emit("progress", "DISPATCHER_EXECUTION_BUILDING_PAYLOAD", block.block_number);
              
              const encodedCalls = baseDecode( buildCalls(validatedCalls) );
              payload = new Uint8Array([
                ...payload,
                ...padSize(encodedCalls.length),
                ...padSize(call.call_index),
                ...encodedCalls,
                ...chainOriginPublicKey,
              ]);
            } catch(e) {
              // encrypted message
              // send directly to enclave without validation
              const encryptedCall = hexToU8a(call.encoded);
              payload = new Uint8Array([
                ...payload,
                ...padSize(encryptedCall.length),
                ...padSize(call.call_index),
                ...encryptedCall,
                ...chainOriginPublicKey
              ]);
            }
          }

          if (this.#progress) this.#progress.emit("progress", "DISPATCHER_EXECUTION_DISPATCHING", block.block_number);

          const outcomes = await this.callRuntime(payload, latestStateRoot);
          let callOutcomeOffset = 0;

          const executedCallIndexes = [];
          while(callOutcomeOffset < outcomes.length) {
            const outcomeSize = unpadSize(outcomes.slice(callOutcomeOffset, callOutcomeOffset + 4));

            if (outcomeSize === 0) {
              // TODO: when would this happen? Empty blocks won't be sent for execution
              break;
            }

            const rawOutcome = outcomes.slice(callOutcomeOffset + 4, callOutcomeOffset + 4 + outcomeSize);
            const outcome = parseOutcomes( baseEncode( rawOutcome) );
            executedCallIndexes.push(outcome.call_id);
            this.#query += this.#db.createOutcome(outcome.call_id, rawOutcome);
            callOutcomeOffset += 4 + outcomeSize;
            latestStateRoot = outcome.state_root;
          }
          this.#query += this.#db.updateOutcomesToBlock(block.block_number, shardId, executedCallIndexes);
          this.#query += this.#db.updateStateRoot(block.block_number, shardId, latestStateRoot);

          await this.writeAll();
          if (this.#progress) this.#progress.emit("progress", "DISPATCHER_EXECUTION_DONE", block.block_number);
        }
      }
    }
  }

  public async preProcessCall(shardId: number, call: Call): Promise<Call> {
    const originAddress = encodeAddress(call.origin_public_key);
    switch(call.transaction_action) {
      // 0. if its an account creation call
      case 0:  
        
        if (originAddress !== '5EYCAe5jKbSe4DzkVVriG3QW13WG4j9gy4zmUxjqT8czBuyu') {
          throw new Error("Wrong Origin - this should never happen");
        }

        return call;

      // 1. if its a transfer action
      case 1:
        // NOT ALLOWED!
        return null;

      // 3. if its a function call
      case 2:
        // Do nothing
        return call;

      // 4. if its a view method call
      case 3: 
        return call;

      // 4. if its a smart contract deployment call
      case 4:
        // 0.1 Verification
        if (originAddress !== '5EYCAe5jKbSe4DzkVVriG3QW13WG4j9gy4zmUxjqT8czBuyu') {
          throw new Error("Wrong Origin - this should never happen");
        }

        // 0.2 Fetch contract 
        const contract = await this.#db.selectWasmBlob(shardId, u8aToString(call.contract_name));
        // 0.3 Put the call to res
        let res: Call = call;
        res.wasm_code = contract;
        return res;
      default: 
        return null;
    }
  }

  public async callRuntime(encodedBlock: Uint8Array, stateRoot: Uint8Array): Promise<Uint8Array> {  
    await initEnclave(bridgeConfig.stateDumpPrefix, stateRoot);
    return await callEnclave(encodedBlock, stateRoot);
  }
}
