import { hexToU8a } from '@polkadot/util';
import { u8aToHex } from '@skyekiwi/util';
import Surreal from 'surrealdb.js';
import { BuffedTx, ExpandedBlockForCall, ExpandedBlockForOutcomes, ShardInfo } from '../types';
import bridgeConfig from '../config';

export class DB {

  #db: Surreal;
  isInit: boolean

  constructor() {
    this.#db = new Surreal(bridgeConfig.surrealDBEndpoint);
  }

  public isReady(): boolean {
    return this.isInit;
  }
  public async init() {
    await this.#db.signin({ user: 'root', pass: 'root', });
    await this.#db.use('skw', 'data');
    this.isInit = true;
    // maybe init metadata
  }

  public async modify(sql: string) {
    return await this.#db.query(sql);
  }

  public async query(sql: string) {
    const res = await this.#db.query(sql);
    return res[0].result;
  }

  public createCall(callIndex: number, encodedCalls: Uint8Array, origin: string) {
    return `
      CREATE call:${callIndex} SET \
        encoded = "${u8aToHex(encodedCalls)}", \
        call_index = ${callIndex}, \
        origin = "${origin}";
    `;
  }

  public createOutcome(callIndex: number, encodedOutcomes: Uint8Array) {
    return `
      CREATE outcome:${callIndex} SET \
        encoded = "${u8aToHex(encodedOutcomes)}", \
        call_index = ${callIndex};
    `;
  }

  // BLOCK
  public createBlock(blockNumber: number, shardId: number) {
    return `
      CREATE block SET \
        block_number = ${blockNumber}, \
        shard_id = shard:${shardId}, \
        calls = [], \
        state_root = "", \
        outcomes = [], 
        status = "Pending";
    `;
  }

  public updateStateRoot(blockNumber: number, shardId: number, stateRoot: Uint8Array) {
    // this is the last step for executing a block - we set status to ReadyForSubmission here
    return `
      UPDATE block \
        SET state_root = "${u8aToHex(stateRoot)}", 
            status = "ReadyForSubmission"
      WHERE block_number = ${blockNumber} && shard_id = shard:${shardId};
    `
  }

  public updateAllBlockStatusAsSynced(shardId: number) {
    return `
      UPDATE block \
        SET status = "Synced"
      WHERE shard_id = shard:${shardId};
    `
  }

  public async selectBlocksForExecution(shardId: number) {
    const res = (await this.query(`
      SELECT * FROM block WHERE \
        shard_id = shard:${shardId} && outcomes = [] \
        ORDER BY block_number ASC
        FETCH calls;
    `)) as ExpandedBlockForCall[];

    if ( res ) {
      return res;
    } else {
      return null;
    }
  }

  public async selectBlocksForSubmission(shardId: number) {
    const res = (await this.query(`
      SELECT * FROM block WHERE \
        shard_id = shard:${shardId} && submitted = "ReadyForSubmission" \
        ORDER BY block_number ASC
        FETCH outcomes;
    `)) as ExpandedBlockForOutcomes[];

    if ( res ) {
      return res;
    } else {
      return null;
    }
  }

  public updateCallsToBlock(blockNumber: number, shardId: number, calls: number[]) {
    return `
      UPDATE block \
        SET calls = [ ${ calls.map(callIndex => `call:${callIndex}`) } ]
      WHERE block_number = ${blockNumber} && shard_id = shard:${shardId};
    `
  }

  public updateOutcomesToBlock(blockNumber: number, shardId: number, calls: number[]) {
    return `
      UPDATE block \
        SET outcomes = [ ${ calls.map(callIndex => `outcome:${callIndex}`) } ]
      WHERE block_number = ${blockNumber} && shard_id = shard:${shardId};
    `
  }

  public async getHighIndexedBlock() {
    const res = (await this.query(`
      SELECT math::max(block_number) as block_number \
        FROM block WHERE calls != [] \
        GROUP BY block_number;
    `)) as {[key: string]: number}[];

    if (res && res[0] && res[0].hasOwnProperty("block_number") ) {
      return res[0]["block_number"] as number;
    } else {
      return null;
    }
  }

  public async getHighExecutedBlock() {
    const res = (await this.query(`
      SELECT math::max(block_number) as block_number \
        FROM block WHERE outcomes != [] \
        GROUP BY block_number;
    `)) as {[key: string]: number}[];;

    if (res && res[0] && res[0].hasOwnProperty("block_number") ) {
      return res[0]["block_number"] as number;
    } else {
      return null;
    }
  }

  public async getLatestStateRoot(shardId: number) {
    // TODO: there might be a better way of doing this
    const res = (await this.query(`
      SELECT state_root FROM block \
        WHERE shard_id = shard:${shardId} && outcomes != [] \
        ORDER BY block_number DESC \
        LIMIT 1;
    `)) as {[key: string]: string}[];;

    if (res && res[0] && res[0].hasOwnProperty("state_root") ) {
      return hexToU8a( res[0]["state_root"] as string );
    } else {
      return new Uint8Array(32);
    }
  }

  // SHARD
  public createShard(
    shardId: number, 
  ) {
    return `
      CREATE shard:${shardId} SET \
        shard_members = [], \
        beacon_index = 0, \
        threshold = 0;
    `;
  }

  public updateShard(
    shardId: number, 
    shardMembers: string[],
    beaconIndex: number,
    threshold: number,
  ) {
    return `
      UPDATE shard:${shardId} SET \
        shard_members = ${shardMembers}, \
        beacon_index = ${beaconIndex}, \
        threshold = ${threshold};
    `;
  }

  public async selectShard(shardId: number) {
    const res = (await this.query(`SELECT * FROM shard:${shardId};`)) as ShardInfo[];
    return res[0];
  }

  // BUFFED TX
  public createTxBuffer(encodedTx: string) {
    return `
      CREATE tx_buffer SET \
        encoded_tx = "${encodedTx}", \
        status = "Pending";
    `;
  }

  public async getPendingTxFromTxBuffer() {
    return (await this.query(`
      SELECT * FROM tx_buffer \
        WHERE status = "Pending";
    `)) as BuffedTx[];
  }

  public updateTxBufferToResolved() {
    return `
      UPDATE tx_buffer SET status = "Resolved";
    `
  }

  // WASM BLOB
  public createWasmBlob(shardId: number, contractName: string, wasmBlob: string) {
    return `
      CREATE wasm_blob SET \
        shard_id = ${shardId}, \
        contract_name = "${contractName}", \
        wasm_blob_bytes = "${wasmBlob}";
    `;
  }

  public async selectWasmBlob(shardId: number, contractName: string) {
    const res = (await this.query(`
      SELECT wasm_blob_bytes \
        FROM wasm_blob \
        WHERE shard_id = ${shardId} && contract_name = ${contractName};
    `)) as {[key: string]: string}[];

    if (res && res[0] && res[0].hasOwnProperty("block_number") ) {
      return hexToU8a( res[0]["wasm_blob_bytes"] );
    } else {
      return null;
    }
  }

  // INDEXER METADATA
  // We skip empty blocks - therefore, might be better to maintain another metadata 
  // to record the last indexed block to reduce repeated request.
  public createIndexerMetadata() {
    return `
      CREATE indexer_metadata SET high_indexed_block = 0;
    `;
  }

  public updateIndexerMetadata(blockNumber: number) {
    return `
      UPDATE indexer_metadata SET high_indexed_block = ${blockNumber};
    `;
  }

  public async selectIndexerMetadata() {
    const res = (await this.query(`
      SELECT high_indexed_block FROM indexer_metadata;
    `)) as {[key: string]: number}[];

    if (res && res[0] && res[0].hasOwnProperty("high_indexed_block") ) {
      return res[0]["high_indexed_block"] as number;
    } else {
      return null;
    }
  }

  // MISC
  public async commitBlockQuery(query: string) {
    const blockQuery = `
      BEGIN TRANSACTION; \
      ${query}
      COMMIT TRANSACTION;
    `;

    const res = await this.modify(blockQuery);

    for (const r of res) {
      // @ts-ignore
      if (r.status !== "OK") {
        console.error(r);
      }
    }
  }
}
