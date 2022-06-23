// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { ApiPromise } from '@polkadot/api';
import type { CallRecord } from './types';

import level from 'level';
import { getLogger, hexToU8a } from '@skyekiwi/util';
import { 
  ExecutionSummary, ShardMetadata, Block, LocalMetadata, parseCalls, baseEncode,
} from '@skyekiwi/s-contract/borsh';

import { DBOps } from './types';
import { Storage } from './storage';

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Indexer {
  #db: level.LevelDB
  #ops: DBOps[]
  #active: boolean

  constructor(db: level.LevelDB) {
    this.#db = db;
    this.#ops = []
    this.#active = true;
  }

  public async done() {
    await this.writeAll();
    this.#active = false;
    await this.#db.close();
  }

  public async initialzeLocalDatabase (): Promise<void> {
    const localMetadata = new LocalMetadata({
      shard_id: [0],
      high_local_block: 0,
    });

    const executionSummary = new ExecutionSummary({
      high_local_execution_block: 0,
      latest_state_root: new Uint8Array(32),
    });

    this.#ops.push(Storage.writeMetadata(localMetadata));
    this.#ops.push(Storage.writeExecutionSummary(executionSummary));

    await Storage.writeAll(this.#db, this.#ops);
    this.#ops = [];
  }

  public async fetchAll (api: ApiPromise, start?: number): Promise<void> {
    const logger = getLogger('indexer.fetchAll');

    const localMetadata = await Storage.getMetadata(this.#db);
    const highLocalBlock = localMetadata.high_local_block ? localMetadata.high_local_block : 0;

    start = start ? start : highLocalBlock;

    let fetchingCount = 0;

    logger.info(`💁 highest local block at ${localMetadata.high_local_block}`);
    let currentBlockNumber = highLocalBlock;

    while (true && this.#active) {
      logger.debug(`⬇️ fetching all info from block# ${currentBlockNumber}`);

      for (const shardId of localMetadata.shard_id) {
        await this.fetchCalls(api, shardId, currentBlockNumber);
      }

      currentBlockNumber++;
      const currentHighBlockNumber = Number((await api.query.system.number()).toJSON());

      if (isNaN(currentHighBlockNumber) || currentBlockNumber >= currentHighBlockNumber) {
        logger.info(`✅ all catchuped ... for now at block# ${currentHighBlockNumber}`);

        const localMetadata = await Storage.getMetadata(this.#db);
        localMetadata.high_local_block = currentHighBlockNumber - 1;

        this.#ops.push(Storage.writeMetadata(localMetadata));
        break;
      }

      // if we have fetched 1000 blocks - write to DB NOW!
      fetchingCount = fetchingCount + 1;
      if (fetchingCount === 1000) {
        fetchingCount = 0;
        logger.info("💯 writting some buffered blocks to local DB")

        const localMetadata = await Storage.getMetadata(this.#db);
        localMetadata.high_local_block = currentBlockNumber;
        this.#ops.push(Storage.writeMetadata(localMetadata));
        await this.writeAll();
      }
    }
  }

  public async fetchCalls (api: ApiPromise, shardId: number, blockNumber: number): Promise<boolean> {
    const logger = getLogger('indexer.fetchCalls');

    const calls = (await api.query.sContract.callHistory(shardId, blockNumber)).toJSON() as number[];
    if (calls) {
      for (const call of calls) {
        const callContent = (await api.query.sContract.callRecord(call)).toJSON() as CallRecord;
        const rawCall = hexToU8a(callContent[0].substring(2)); // trim out the '0x'

        // TODO: we no longer force base64 encoding of calls anymore
        this.#ops.push(Storage.writeCallsRecord(shardId, call, 
            parseCalls( baseEncode(rawCall) )
          )
        );
      }
    }

    const block = new Block({
      shard_id: shardId,
      block_number: blockNumber,
      calls: calls,
    });
    this.#ops.push(Storage.writeBlockRecord(shardId, blockNumber, block));
    logger.info(`📦 block import complete at block# ${blockNumber}, imported ${calls ? calls.length : 0} calls`);
    return true;
  }

  public async fetchShardInfo (api: ApiPromise, address: string): Promise<void> {
    const localMetadata = await Storage.getMetadata(this.#db);

    for (const shard of localMetadata.shard_id) {
      const maybeMembers = await api.query.registry.shardMembers(shard);
      const maybeBeaconIndex = await api.query.registry.beaconIndex(shard, address);
      const maybeThreshold = await api.query.parentchain.shardConfirmationThreshold(shard);

      const members = maybeMembers.toJSON();
      const beaconIndex = Number(maybeBeaconIndex.toString());
      const threshold = Number(maybeThreshold.toString());

      let shardInfo;

      try {
        shardInfo = await Storage.getShardMetadataRecord(this.#db, shard);

        // shard has been recorded in the system
        // Updating ...
        shardInfo.shard_members = members as string[];
        shardInfo.beacon_index = beaconIndex;
        shardInfo.threshold = threshold || threshold === 0 ? 1 : threshold;
      } catch (e) {
        // shard has not been recorded in the system
        // Recording ...
        shardInfo = new ShardMetadata({
          // TODO: shard_key 
          shard_key: new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
          shard_members: members as string[],
          beacon_index: beaconIndex,
          threshold: threshold
        });
      }

      // TODO: better handle of this - we do not force registration on genesis anymore
      if (!shardInfo.shard_members) {
        shardInfo.shard_members = []
      }

      this.#ops.push(Storage.writeShardMetadataRecord(shard, shardInfo));
    }
  }

  public async writeAll (): Promise<void> {
    if (this.#active) {
      await Storage.writeAll(this.#db, this.#ops);
      this.#ops = []
    }
  }
}
