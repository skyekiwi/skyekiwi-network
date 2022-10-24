import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import type { BlockRawCalls, ShardInfo, CallRecord } from "../types";

import { ApiPromise } from "@polkadot/api";
import { hexToU8a, u8aToHex } from '@skyekiwi/util';

export class Chain {
  #api: ApiPromise

  constructor (api: ApiPromise) {
    this.#api = api;
  }

  // Raw Blockchain Interaction

  // QUERY
  public async queryBlockNumber(): Promise<number> {
    return Number((await this.#api.query.system.number()).toString())
  }

  public async queryPreimage(hash: string): Promise<string> {
    return (await this.#api.query.preimage.preimageFor(hash)).toString();
  }
  
  public async querySContractCallHistory(shardId: number, blockNumber: number): Promise<number[]> {
    return (await this.#api.query.sContract.callHistory(shardId, blockNumber)).toJSON() as number[];
  }

  public async querySContractCallRecord(callIndex: number): Promise<CallRecord> {
    return (await this.#api.query.sContract.callRecord(callIndex)).toJSON() as CallRecord;
  }
  
  public async querySContractNextCallIndex(): Promise<number> {
    return Number((await this.#api.query.sContract.currentCallIndex()).toString());
  }

  public async querySContractWasmBlob(shardId: number, contractName: string): Promise<string> {
    return (await this.#api.query.sContract.wasmBlob(shardId, contractName)).toString();
  }

  public async queryRegistryExpiration(address: string): Promise<number> {
    return Number((await this.#api.query.registry.expiration(address)).toString())
  }

  public async queryRegistryShardMembers(shardId: number): Promise<string[]> {
    return (await this.#api.query.registry.shardMembers(shardId)).toJSON() as string[];
  }

  public async queryParentchainConfirmationThreshold(shardId: number): Promise<number> {
    return Number((await this.#api.query.parentchain.shardConfirmationThreshold(shardId)).toString());
  }

  public async queryRegistryBeaconIndex(shardId: number, address: string): Promise<number> {
    return Number((await this.#api.query.registry.beaconIndex(shardId, address)).toString());
  }

  public async queryParentchainConfirmation(shardId: number, blockNumber: number): Promise<number> {
    return Number((await this.#api.query.parentchain.confirmation(shardId, blockNumber)).toString());
  }

  public async queryParentchainOutcome(callIndex: number): Promise<string> {
    return (await this.#api.query.parentchain.outcome(callIndex)).toJSON() as unknown as string;
  }

  // TX
  public txBatch(exts: SubmittableExtrinsic[]): SubmittableExtrinsic {
    return this.#api.tx.utility.batch(exts)
  }

  public txRegisteryRegisterSecretKeeper(publicKey: Uint8Array, singature: Uint8Array): SubmittableExtrinsic {
    return this.#api.tx.registry.registerSecretKeeper( "0x" + u8aToHex(publicKey), "0x" + u8aToHex( singature ) )
  }

  public txRegisteryRegisterRunningShard(shardId: number): SubmittableExtrinsic {
    return this.#api.tx.registry.registerRunningShard(shardId)
  }

  public txRegisteryRenewRegistration(publicKey: Uint8Array, singature: Uint8Array): SubmittableExtrinsic {
    return this.#api.tx.registry.renewRegistration( "0x" + u8aToHex(publicKey), "0x" + u8aToHex( singature ) )
  }

  public txParentchainSubmitOutcome(
    blockNumber: number, shardId: number,
    stateRoot: Uint8Array, 
    callIndexs: number[], outcomes: string[]
  ): SubmittableExtrinsic {
    return this.#api.tx.parentchain.submitOutcome(
      blockNumber, shardId, stateRoot,
      callIndexs, outcomes
    )
  }

  public encodedTxToBatchSubmittable(encoded: string[]): SubmittableExtrinsic {
    if (!encoded || encoded.length === 0) return null;
    return this.txBatch( encoded.map(e => this.#api.tx(e)) )
  }

  // Derived Blockchain Interaction
  public async getWasmBlob(shardId: number, contractName: string): Promise<string> {
    const hash = await this.querySContractWasmBlob(shardId, contractName);
    const content = await this.queryPreimage(hash);

    return content
  }

  public async getRawCalls(shardId: number, blockNumber: number): Promise<BlockRawCalls> {
    const res = [];
    const history = await this.querySContractCallHistory(shardId, blockNumber);
    if (history) {
      for (const callIndex of history) {
        const call = await this.querySContractCallRecord(callIndex);
        res.push([callIndex, hexToU8a(call[0].substring(2)), call[1]]);
      }
    }

    return res as BlockRawCalls
  }

  public async getShardMetadata(shardId: number, address: string): Promise<ShardInfo> {
    const members = await this.queryRegistryShardMembers(shardId);
    const beaconIndex = await this.queryRegistryBeaconIndex(shardId, address);
    const threshold = await this.queryParentchainConfirmationThreshold(shardId);

    return {
      shard_member: members,
      beacon_index: beaconIndex,
      threshold: threshold || threshold === 0 ? 1 : threshold,
    } as ShardInfo;
  }

  public async maybeRegistSecretKeeper(
    blockNumber: number, 
    address: string, activeShards: number[],
    publicKey: Uint8Array, signature: Uint8Array
  ): Promise<SubmittableExtrinsic[]> {
    const expiration = await this.queryRegistryExpiration(address);

    const buf: SubmittableExtrinsic[] = [];

    if (!expiration) {
      buf.push( this.txRegisteryRegisterSecretKeeper(publicKey, signature) );
    } else if (expiration - 10 < blockNumber) {
      buf.push( this.txRegisteryRenewRegistration(publicKey, signature) );
    } else {
      return buf;
    }

    for (const shard of activeShards) {
      buf.push(this.txRegisteryRegisterRunningShard(shard));
    }

    return buf;
  }
  
}