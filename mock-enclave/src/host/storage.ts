// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { DBOps } from './types';

import level from 'level';

import { 
  Calls, buildCalls, parseCalls,
  Outcomes, buildOutcomes, parseOutcomes,
  Block, buildBlock, parseBlock,
  Shard, buildShard, parseShard,
  ShardMetadata, buildShardMetadata, parseShardMetadata,
  LocalMetadata, buildLocalMetadata, parseLocalMetadata,
  ExecutionSummary, buildExecutionSummary, parseExecutionSummary,
  BlockSummary, buildBlockSummary, parseBlockSummary,
} from '@skyekiwi/s-contract/borsh';

const numberPadding = (n: number, pad: number): string => {
  return String(n).padStart(pad, '0');
};

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Storage {
  public static getCallsIndex (shardId: number, callIndex: number): string {
    const shard = numberPadding(shardId, 4);
    const block = numberPadding(callIndex, 16);

    return shard + block + 'RAWC';
  }

  public static getCallOutcomeIndex (shardId: number, callIndex: number): string {
    const shard = numberPadding(shardId, 4);
    const block = numberPadding(callIndex, 16);

    return shard + block + 'OUTC';
  }

  public static getBlockIndex (shardId: number, blockNumber: number): string {
    const shard = numberPadding(shardId, 4);
    const block = numberPadding(blockNumber, 16);

    return shard + block + 'BLOC';
  }

  public static getBlockSummaryIndex (shardId: number, blockNumber: number): string {
    const shard = numberPadding(shardId, 4);
    const block = numberPadding(blockNumber, 16);

    return shard + block + 'BSUM';
  }

  public static getShardIndex (shardId: number): string {
    return numberPadding(shardId, 20) + 'SHAR';
  }

  public static getShardMetadataIndex (shardId: number): string {
    return numberPadding(shardId, 20) + 'SHAM';
  }

  public static writeMetadata (
    metadata: LocalMetadata
  ): DBOps {
    return {
      type: 'put',
      key: 'METADATA',
      value: buildLocalMetadata(metadata)
    };
  }

  public static async getMetadata (db: level.LevelDB): Promise<LocalMetadata> {
    return parseLocalMetadata(await db.get('METADATA'));
  }

  public static writeCallsRecord (shard_id: number, callIndex: number, call: Calls): DBOps {
    const key = Storage.getCallsIndex(shard_id, callIndex);

    return {
      type: 'put',
      key: key,
      value: buildCalls(call)
    };
  }

  public static writeCallOutcome (shardId: number, callIndex: number, outcome: Outcomes): DBOps {
    const key = Storage.getCallOutcomeIndex(shardId, callIndex);

    return {
      type: 'put',
      key: key,
      value: buildOutcomes(outcome)
    };
  }

  public static writeBlockRecord (shardId: number, blockNumber: number, block: Block): DBOps {
    const key = Storage.getBlockIndex(shardId, blockNumber);

    return {
      type: 'put',
      key: key,
      value: buildBlock(block)
    };
  }

  public static writeShardRecord (shardId: number, shard: Shard): DBOps {
    const key = Storage.getShardIndex(shardId);

    return {
      type: 'put',
      key: key,
      value: buildShard(shard)
    };
  }

  public static writeShardMetadataRecord (shardId: number, shardm: ShardMetadata): DBOps {
    const key = Storage.getShardMetadataIndex(shardId);

    return {
      type: 'put',
      key: key,
      value: buildShardMetadata(shardm)
    };
  }

  public static writeBlockSummary (shardId: number, blockNumber: number, blockS: BlockSummary): DBOps {
    const key = Storage.getBlockSummaryIndex(shardId, blockNumber);

    return {
      type: 'put',
      key: key,
      value: buildBlockSummary(blockS)
    };
  }

  public static writeExecutionSummary (a: ExecutionSummary): DBOps {
    return {
      type: 'put',
      key: 'EXECUTION_SUMMARY',
      value: buildExecutionSummary(a)
    };
  }

  public static async writeAll (db: level.LevelDB, ops: DBOps[]): Promise<void> {
    // eslint-diable
    // @ts-ignore
    await db.batch(ops);
    // eslint-enable
  }

  public static async getCallsRecord (
    db: level.LevelDB, shardId: number, callIndex: number
  ): Promise<Calls> {
    const key = Storage.getCallsIndex(shardId, callIndex);
    const c = parseCalls(await db.get(key));

    let res = new Calls({
      ops: [],
      shard_id: c.shard_id,
      block_number: c.block_number,
    });

    for (let op of c.ops) {
      let convertedOp = op;
      convertedOp.origin_public_key = new Uint8Array(convertedOp.origin_public_key);
      convertedOp.receipt_public_key = new Uint8Array(convertedOp.receipt_public_key);
      if (convertedOp.contract_name !== undefined){
        convertedOp.contract_name = new Uint8Array(convertedOp.contract_name);
      }

      res.ops.push(convertedOp)
    }


    return res;
  }

  public static async getOutcomesRecord (
    db: level.LevelDB, shardId: number, callIndex: number
  ): Promise<Outcomes> {
    const key = Storage.getCallOutcomeIndex(shardId, callIndex);

    return parseOutcomes(await db.get(key));
  }

  public static async getBlockRecord (
    db: level.LevelDB, shardId: number, blockNumber: number
  ): Promise<Block> {
    const key = Storage.getBlockIndex(shardId, blockNumber);

    return parseBlock(await db.get(key));
  }

  public static async getShardRecord (
    db: level.LevelDB, shardId: number
  ): Promise<Shard> {
    const key = Storage.getShardIndex(shardId);

    return parseShard(await db.get(key));
  }

  public static async getShardMetadataRecord (
    db: level.LevelDB, shardId: number
  ): Promise<ShardMetadata> {
    const key = Storage.getShardMetadataIndex(shardId);

    return parseShardMetadata(await db.get(key));
  }

  public static async getExecutionSummary (
    db: level.LevelDB
  ): Promise<ExecutionSummary> {
    return parseExecutionSummary(await db.get('EXECUTION_SUMMARY'));
  }

  public static async getBlockSummary (
    db: level.LevelDB, shardId: number, blockNumber: number
  ): Promise<BlockSummary> {
    const key = Storage.getBlockSummaryIndex(shardId, blockNumber);
    return parseBlockSummary(await db.get(key));
  }
}
