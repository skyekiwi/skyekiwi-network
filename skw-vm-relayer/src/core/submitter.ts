// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import EventEmitter from "events";

import { hexToU8a, sleep, sendTx } from '@skyekiwi/util';
import { KeyringPair } from '@polkadot/keyring/types';

import { Chain, DB } from '../util';

/* eslint-disable sort-keys, camelcase */
export class Submitter {
  #active: boolean
  #query: string
  #db: DB
  #shards: number[]
  #chain: Chain
  #key: KeyringPair
  #progress?: EventEmitter

  constructor(db: DB, chain: Chain, key: KeyringPair, progress?: EventEmitter) {
    if (!db.isReady()) {
      throw new Error("db not connected");
    }
    this.#query = "";
    this.#active = true;
    this.#db = db;
    this.#shards = []
    this.#chain = chain;
    this.#key = key;
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

  public async parseAllSubmission() {
    while(this.#active) {
      await this.keepSecretKeeperRegistered();

      for (const shardId of this.#shards) {
        const blocks = await this.#db.selectBlocksForSubmission(shardId);
        if (!blocks || blocks.length === 0) {
          if (this.#progress) this.#progress.emit("progress", "SUBMITTER_SKIP_SUBMISSION");
          // Sleep for a block time
          sleep(6000);
        }

        for (const block of blocks) {
          const callIndexes = [];
          const outcomes = [];

          for (const outcome of block.outcomes) {
            callIndexes.push(outcome.call_index);
            outcomes.push(outcome.encoded);
          }

          const tx = this.#chain.txParentchainSubmitOutcome(
            block.block_number, shardId, hexToU8a( block.state_root ),
            callIndexes, outcomes
          );

          this.#query += this.#db.createTxBuffer(tx.toHex());

          if (this.#progress) this.#progress.emit("progress", "SUBMITTER_BLOCK_GENERATED", block.block_number);
          this.#query += this.#db.updateAllBlockStatusAsSubmitted(shardId, block.block_number);
        }

        await this.writeAll();
        await this.submitAllTx();
        if (this.#progress) this.#progress.emit("progress", "SUBMITTER_SUBMITTED");

        await sleep(4000);
      }
    }
  }

  public async keepSecretKeeperRegistered() {
    const blockNumber = await this.#chain.queryBlockNumber();
    // keep secret keeprer registered FIRST
    const fakePublicKey = "1111111111111111111111111111111111111111111111111111111111111111"
    const fakeSignature = "11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"

    const register = await this.#chain.maybeRegistSecretKeeper(
      blockNumber, this.#key.address, this.#shards, 
      hexToU8a( fakePublicKey ), hexToU8a( fakeSignature )
    );

    if (register && register.length !== 0) {
      this.#query += this.#db.createTxBuffer(
        this.#chain.txBatch(register).toHex()
      )
    }

    await this.writeAll();
    await this.submitAllTx();
  }

  public async submitAllTx() {
    const buffedTx = await this.#db.getPendingTxFromTxBuffer();
    const submittable = this.#chain.encodedTxToBatchSubmittable(
      buffedTx.map(t => t.encoded_tx)
    );
    if (submittable) await sendTx(submittable, this.#key);

    this.#query += this.#db.updateTxBufferToResolved();
    await this.writeAll();
  }
}
