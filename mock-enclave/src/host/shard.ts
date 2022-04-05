// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { randomBytes } from 'tweetnacl';
import Level from 'level';

import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/keyring'

import { AsymmetricEncryption } from '@skyekiwi/crypto';
import { ShardMetadata, buildOutcomes } from '@skyekiwi/s-contract/borsh';
import { getLogger, sendTx, u8aToHex } from '@skyekiwi/util';

import {Storage} from './storage'
import { SubmittableExtrinsic } from '@polkadot/api/promise/types';
import { QueuedTransaction } from './types';

export class ShardManager {
  #keyring: KeyringPair

  #key: Uint8Array
  #shards: number[]

  constructor (runningShards: number[], key?: Uint8Array) {
    this.#shards = runningShards;
    this.#key = key || randomBytes(32);
  }

  public async init (): Promise<void> {
    const seed = process.env.TEST_SEED_PHRASE;

    if (!seed) {
      throw new Error('seed phrase not found');
    }

    await waitReady();
    this.#keyring = new Keyring({ type: 'sr25519' }).addFromUri(seed);
  }

  public async maybeRegisterSecretKeeper (api: ApiPromise, blockNumber: number): Promise<SubmittableExtrinsic[]> {
    const logger = getLogger(`shardManager.maybeRegisterSecretKeeper`); 
    const allExtrinsics = [];

    const maybeExpiration = await api.query.registry.expiration(this.#keyring.address);
    const expiration = Number(maybeExpiration.toString());

    if (isNaN(expiration) || expiration - 10 < blockNumber) {
      logger.info(`registering secret keeper at blockNumber ${blockNumber}`);

      // not previously registered
      allExtrinsics.push(api.tx.registry.registerSecretKeeper(
        u8aToHex(AsymmetricEncryption.getPublicKey(this.#key)),
        '0x0000'
      ));

      for (const shard of this.#shards) {
        allExtrinsics.push(api.tx.registry.registerRunningShard(shard));
      }

      return allExtrinsics;
    }
    
    return null
  }

  public async maybeSubmitExecutionReport (api: ApiPromise, db: Level.LevelDB, blockNumber: number, fileHash: Uint8Array): Promise<SubmittableExtrinsic[]> {
    const logger = getLogger(`shardManager.maybeSubmitExecutionReport`); 

    const tx = [];
    for (const shard of this.#shards) {

      const shardMetadata = await Storage.getShardMetadataRecord(db, shard);
      if (this.beaconIsTurn(blockNumber, shardMetadata)) {

        logger.info(`in turn and submitting executing report for blockNumber ${blockNumber}`);

        const block = await Storage.getBlockRecord(db, shard, blockNumber);

        if (!block.calls || !block.contracts) {
          console.log("unexpected", block, shard)
          continue;
        }

        let stateRoot: Uint8Array
        let outcomes: string[] = []
        let callIndex: number[] = []

        for (const call of block.calls) {
          const o = await Storage.getOutcomesRecord(db, shard, call);
          outcomes.push(buildOutcomes(o));
          callIndex.push(call);
          stateRoot = o.state_root;
        }

        tx.push(
          api.tx.parentchain.submitOutcome(
            blockNumber, 0, stateRoot, fileHash,
            callIndex, outcomes
          )
        )
      }
    }

    return tx;
  }

  // if curBlockNumber is undefined -> forceSubmitAllTx
  public async maybeSubmitTxBatch(api: ApiPromise, buffer: QueuedTransaction[], curBlockNumber?: number): Promise<QueuedTransaction[]> {    
    const submissionFilter = (it: QueuedTransaction) => {
      if (!curBlockNumber) return true;
      return it.blockNumber < curBlockNumber - 5 && it.blockNumber !== -1
    };

    let tx: SubmittableExtrinsic[] = buffer
      .filter(it => submissionFilter(it))
      .map(it => it.transaction)
    await sendTx(api.tx.utiliy.batchAll(tx), this.#keyring);
    
    let res = buffer.filter(it => !submissionFilter(it))
    if (res.length === 0) {
      res = [{
        transaction: null,
        blockNumber: -1
      }]
    }

    return res;
  }

  private beaconIsTurn (
    blockNumber: number, shard: ShardMetadata
  ): boolean {

    const beaconIndex = shard.beacon_index;
    const threshold = shard.threshold;
    const beaconCount = shard.shard_members.length;

    // 1 2 3 4 5 6 7 8 9
    return threshold >= beaconCount ||
      (
    // _ X X X _ _ _ _ _
        blockNumber % beaconCount <= beaconIndex &&
          beaconIndex <= blockNumber % beaconCount + threshold - 1
      ) ||
      (
    // X X _ _ _ _ _ _ X
        blockNumber % beaconCount + threshold - 1 > beaconCount &&
          (
            beaconCount - (blockNumber % beaconCount + threshold - 1) % beaconCount <= beaconIndex ||
              beaconIndex <= blockNumber % beaconCount + threshold - 1 - beaconCount
          )
      );
  }
}
