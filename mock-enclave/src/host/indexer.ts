// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { ApiPromise } from '@polkadot/api';
import type { EventRecord } from '@polkadot/types/interfaces';
import type { CallRecord } from './types';

import level from 'level';
import { getLogger, hexToU8a, u8aToString } from '@skyekiwi/util';
import { 
  Calls, ExecutionSummary, ShardMetadata, Block, LocalMetadata, parseCalls, Contract, Outcomes
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
      latest_state_root: new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
    });

    const executionSummary = new ExecutionSummary({
      high_local_execution_block: 0,
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

    logger.info(`highest local block at ${localMetadata.high_local_block}`);
    let currentBlockNumber = highLocalBlock;

    while (true && this.#active) {
      logger.debug(`fetching all info from block# ${currentBlockNumber}`);

      for (const shardId of localMetadata.shard_id) {
        for (let i = 0; i < 10; i ++) {
          const s = await this.fetchCalls(api, shardId, currentBlockNumber);
          if (s) break;

          logger.info(`retrying for ${currentBlockNumber}`)
        }
      }

      currentBlockNumber++;
      const currentHighBlockNumber = Number((await api.query.system.number()).toJSON());

      if (isNaN(currentHighBlockNumber) || currentBlockNumber >= currentHighBlockNumber) {
        logger.info(`all catchuped ... for now at block# ${currentHighBlockNumber}`);

        const localMetadata = await Storage.getMetadata(this.#db);
        localMetadata.high_local_block = currentHighBlockNumber - 1;
        this.#ops.push(Storage.writeMetadata(localMetadata));
        break;
      }

      fetchingCount = fetchingCount + 1;

      if (fetchingCount === 1000) {
        fetchingCount = 0;
        logger.info("writting some calls to local DB")

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
    let contracts: string[] = []

    // 1. build new contract deployments
    const blockHash = ((await api.rpc.chain.getBlockHash(blockNumber)).toJSON()) as string;
    if (blockHash === '0x0000000000000000000000000000000000000000000000000000000000000000')
      return false;
    const events = (await api.query.system.events.at(blockHash)).toHuman() as unknown as EventRecord[];

    for (const evt of events) {
      if (evt.event) {
        if (evt.event.method === 'SecretContractRegistered') {
          const shardId = Number(evt.event.data[0]);
          const contractName = evt.event.data[1].toString();
          const callIndex = Number(evt.event.data[2]);

          let call
          const encodedCall = (await api.query.sContract.callRecord(shardId, callIndex)).toJSON() as CallRecord;
          if (encodedCall[0] === '0x') {
            call = new Calls({ops: [] });
          }
          else {
            call = parseCalls( u8aToString( hexToU8a(encodedCall[0].substring(2)) ) );
          }

          contracts.push(contractName);
          const wasmCIDRaw = (await api.query.sContract.wasmBlobCID(shardId, contractName)).toJSON() as string;
          const wasmCID = u8aToString(hexToU8a(wasmCIDRaw.substring(2)));

          const contract = new Contract({
            home_shard: shardId,
            wasm_blob: wasmCID,
            deployment_call: call,
            deployment_call_index: callIndex,
          });

          this.#ops.push(Storage.writeContractRecord(contractName, contract));
        }
      }
    };

    if (!calls && !contracts) return true;

    // 2. build calls
    if (calls) {
      for (const call of calls) {
        const callContent = (await api.query.sContract.callRecord(shardId, call)).toJSON() as CallRecord;
        let encodedCall
        if (callContent[0] === '0x') {
          encodedCall = ""
        }
        else encodedCall = u8aToString(hexToU8a(callContent[0].substring(2)));
        this.#ops.push(Storage.writeCallsRecord(shardId, call, parseCalls(encodedCall)));
      }
    }

    // 3. build blocks
    const block = new Block({
      shard_id: shardId,
      block_number: blockNumber,
      calls: calls,
      contracts: contracts
    });
    this.#ops.push(Storage.writeBlockRecord(shardId, blockNumber, block));

    logger.info(`block import complete at block# ${blockNumber}, imported ${calls ? calls.length : 0} calls and ${contracts ? contracts.length : 0} contracts`);
    
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
          shard_key: new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
          shard_members: members as string[],
          beacon_index: beaconIndex,
          threshold: threshold
        });
      }

      this.#ops.push(Storage.writeShardMetadataRecord(shard, shardInfo));
    }
  }

  public writeOutcomes(shardId: number, callIndex: number, outcomes: Outcomes): void {
    this.#ops.push(Storage.writeCallOutcome(shardId, callIndex, outcomes));
  }
  public async writeAll (): Promise<void> {
    if (this.#active) {
      await Storage.writeAll(this.#db, this.#ops);
      this.#ops = []
    }
  }
}
