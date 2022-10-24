// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import EventEmitter from "events";

import { baseEncode, parseCalls } from '@skyekiwi/s-contract';
import { encodeAddress } from '@polkadot/util-crypto';
import { u8aToString } from '@polkadot/util';

import { DB, Chain } from '../util';


/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Indexer {
  #active: boolean
  #query: string
  #db: DB
  #shards: number[]
  #currentBlockNumber: number
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
    if (this.#currentBlockNumber) {
      await this.#db.commitBlockQuery(
        this.#db.updateIndexerMetadata(this.#currentBlockNumber)
      );
    }
    if ( this.#active && this.#query ) {
      await this.#db.commitBlockQuery(this.#query);
      this.#query = "";
    }
  }

  public async setShards(shards: number[]) {
    for (const shard of shards) {
      if (!(await this.#db.selectShard(shard))) {
        this.#query += this.#db.createShard(shard);
      }
    }
    
    await this.writeAll();
    this.#shards = shards;
  }

  public async fetch (chain: Chain, start?: number): Promise<void> {
    const highIndexedBlock = await this.#db.selectIndexerMetadata();
    this.#currentBlockNumber = 0;

    if (highIndexedBlock) {
      this.#currentBlockNumber = start ? start : highIndexedBlock + 1;
    }

    let fetchingCount = 0;

    if (this.#progress) this.#progress.emit("progress", "INDEXER_FETCH_START", this.#currentBlockNumber);

    while (this.#active) {
      if (this.#progress) this.#progress.emit("progress", "INDEXER_FETCH_FETCHING", this.#currentBlockNumber);

      for (const shardId of this.#shards) {

        const rawCalls = await chain.getRawCalls(shardId, this.#currentBlockNumber);
        const calls: number[] = []

        for (let [callIndex, call, origin] of rawCalls) {
          calls.push(callIndex);
          const c = parseCalls(baseEncode(call));
          for (let op of c.ops) {
            if (op.transaction_action === 4) {
              const originAddress = encodeAddress(new Uint8Array( op.origin_public_key) );
              if (originAddress !== "5EYCAe5jKbSe4DzkVVriG3QW13WG4j9gy4zmUxjqT8czBuyu") {
                throw new Error("UNEXPECTED! SContract error");
              }
              const contractName = u8aToString(new Uint8Array( op.contract_name) );
              const wasmBlob = await chain.getWasmBlob(shardId, contractName);
              this.#query += this.#db.createWasmBlob(shardId, contractName, wasmBlob);
            }
          }
          this.#query += this.#db.createCall(callIndex, call, origin);
        }; 

        if (calls.length !== 0) {
          this.#query += this.#db.createBlock(this.#currentBlockNumber, shardId);
          this.#query += this.#db.updateCallsToBlock(this.#currentBlockNumber, shardId, calls);
          if (this.#progress) this.#progress.emit("progress", "INDEXER_FETCH_FETCH_BLOCK", this.#currentBlockNumber, calls.length);
        }
      }

      this.#currentBlockNumber ++;
      const highChainBlockNumber = await chain.queryBlockNumber();

      if (isNaN(highChainBlockNumber) || this.#currentBlockNumber === highChainBlockNumber) {
        if (this.#progress) this.#progress.emit("progress", "INDEXER_FETCH_ALLDONE", this.#currentBlockNumber);
        break;
      }

      // if we have fetched 1000 blocks - write to DB NOW!
      fetchingCount = fetchingCount + 1;
      if (fetchingCount === 5000) {
        fetchingCount = 0;
        if (this.#progress) this.#progress.emit("progress", "INDEXER_FETCH_WRITE_BUFFED");
        // logger.info("💯 writting some buffered blocks to local DB")
        await this.writeAll();
      }
    }
  }

  public async fetchShardInfo (chain: Chain, address: string): Promise<void> {
    for (const shard of this.#shards) {
      const newShardInfo = await chain.getShardMetadata(shard, address);
      this.#query += this.#db.updateShard(
        shard, 
        newShardInfo.shard_member, 
        newShardInfo.beacon_index, 
        newShardInfo.threshold
      );
    }
  }
}
