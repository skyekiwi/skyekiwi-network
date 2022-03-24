// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { randomBytes } from 'tweetnacl';

import { AsymmetricEncryption } from '@skyekiwi/crypto';
import { sendTx, u8aToHex } from '@skyekiwi/util';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/keyring'
import Level from 'level';
import {ShardMetadata} from '@skyekiwi/s-contract/borsh'
import {Storage} from './storage'
import { buildOutcomes } from './borsh';

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

  public async maybeRegisterSecretKeeper (api: ApiPromise, blockNumber: number): Promise<void> {
    const allExtrinsics = [];

    const maybeExpiration = await api.query.registry.expiration(this.#keyring.address);
    const expiration = Number(maybeExpiration.toString());

    if (isNaN(expiration) || expiration - 10 < blockNumber) {
      // not previously registered
      allExtrinsics.push(api.tx.registry.registerSecretKeeper(
        u8aToHex(AsymmetricEncryption.getPublicKey(this.#key)),
        '0x0000'
      ));

      for (const shard of this.#shards) {
        allExtrinsics.push(api.tx.registry.registerRunningShard(shard));
      }

      const all = api.tx.utility.batch(allExtrinsics);

      await sendTx(all, this.#keyring);
    }
  }

  public async maybeSubmitExecutionReport (api: ApiPromise, db: Level.LevelDB, blockNumber: number) {
    for (const shard of this.#shards) {

      const shardMetadata = await Storage.getShardMetadataRecord(db, shard);
      if (this.beaconIsTurn(blockNumber, shardMetadata)) {

        const block = await Storage.getBlockRecord(db, shard, blockNumber);
        let stateRoot: Uint8Array
        let outcomes: string[] = []
        let callIndex: number[] = []

        for (const call of block.calls) {
          const o = await Storage.getOutcomesRecord(db, shard, call);
          outcomes.push(buildOutcomes(o));
          callIndex.push(call);
          stateRoot = o.state_root;
        }

        const tx = api.tx.parentchain.submitOutcome(
          blockNumber, 0, stateRoot, new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
          callIndex, outcomes
        )
        await sendTx(tx, this.#keyring);
      }
    }
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
