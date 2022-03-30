import { ApiPromise, WsProvider } from '@polkadot/api';
import level from 'level'

import {Calls, Outcomes} from '@skyekiwi/s-contract/borsh';
import { getLogger} from '@skyekiwi/util';

import { Indexer } from './host/indexer';
import { Storage } from './host/storage'
import { ShardManager } from './host/shard';

import { Dispatcher } from './host/dispatcher';
import { callRuntime  } from './vm';
import { SubmittableExtrinsic } from '@polkadot/api/promise/types';
import config from './config';

import fs from 'fs';
import crypto from 'crypto';

require("dotenv").config()

const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

const processCalls = (calls: Calls, stateRoot: Uint8Array) => {
  if (calls.ops.length === 0) 
    return new Outcomes({
      ops: [],
      state_root: stateRoot,
    });

  const outcomes = callRuntime(calls, stateRoot)
  return outcomes;
}

const getStateFileHash = (): Uint8Array => {
  const file = fs.readFileSync(config.currentStateFile);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(file);
  return hashSum.digest();
}

const main = async () => {

  const logger = getLogger("mock-enclave")

  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider: provider });

  const db = level('local');
  const indexer = new Indexer(db);
  const shard = new ShardManager([0]);
  const dispatcher = new Dispatcher();

  await shard.init();

  try {
    await Storage.getMetadata(db);
  } catch(e) {
    logger.warn(e);
    // local metadata has not been initialized
    await indexer.initialzeLocalDatabase();
  }

  await indexer.fetchShardInfo(api, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');

  let txBuffer: SubmittableExtrinsic[] = [];

  while (true) {
    await indexer.fetchAll(api);
    await indexer.writeAll();

    let executionSummary = await Storage.getExecutionSummary(db);
    let localMetadata = await Storage.getMetadata(db);
    for (let blockNumber = executionSummary.high_local_execution_block === 0 ? 1 : executionSummary.high_local_execution_block; 
      blockNumber < localMetadata.high_local_block; blockNumber ++ ) 
    {
      const block = await Storage.getBlockRecord(db, 0, blockNumber);
      if (blockNumber % 20 === 0) {
        const txs = await shard.maybeRegisterSecretKeeper(api, blockNumber);
        txBuffer.push(... (txs ? txs : []));
      }

      if (dispatcher.isDispatchable(db, blockNumber)) {

        if (block.calls && block.calls.length > 0) {
          for (let call of block.calls) {
            localMetadata.latest_state_root = await dispatcher.dispatchCalls(indexer, db, call, localMetadata.latest_state_root, processCalls)
          }
        }
        
        if (block.contracts && block.contracts.length > 0) {
          for (const contractName of block.contracts) {
            localMetadata.latest_state_root = await dispatcher.dispatchNewContract(indexer, db, contractName, localMetadata.latest_state_root, processCalls);
          }
        }

        executionSummary.high_local_execution_block = block.block_number - 1;

        const op = [
          Storage.writeExecutionSummary(executionSummary),
          Storage.writeMetadata(localMetadata),
        ];
        await Storage.writeAll(db, op);
      }

      await indexer.writeAll();

      const conf = (await api.query.parentchain.confirmation(0, block.block_number)).toJSON();
      if (
          // no reports has been submitted yet || confirmation is below the threshold?
          (conf === null) && (
            (block.calls && block.calls.length !== 0) || 
            (block.contracts && block.contracts.length !== 0)
          ) && dispatcher.isDispatchable(db, block.block_number)
      ) {
        console.log("submitting report for ", block.block_number)
        txBuffer.push(... (await shard.maybeSubmitExecutionReport(api, db, block.block_number, getStateFileHash())));

        if (txBuffer.length >= 100) {
          await shard.submitTxBatch(api.tx.utility.batchAll(txBuffer));
          txBuffer = [];
        }
      }
    }

    await shard.submitTxBatch(api.tx.utility.batchAll(txBuffer));
    txBuffer = [];

    await sleep(6000);
  }
}

main()
