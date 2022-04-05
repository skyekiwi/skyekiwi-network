import { ApiPromise, WsProvider } from '@polkadot/api';
import level from 'level'

import {Calls, Outcomes} from '@skyekiwi/s-contract/borsh';
import { getLogger} from '@skyekiwi/util';

import { Indexer } from './host/indexer';
import { Storage } from './host/storage'
import { ShardManager } from './host/shard';

import { Dispatcher } from './host/dispatcher';
import { callRuntime  } from './vm';
import { validateIndexer, validateExecutor } from './validate';
import config from './config';

import fs from 'fs';
import crypto from 'crypto';
import { QueuedTransaction } from './host/types';

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

const SHUTDOWN_SIGNALS = ['SIGINT', 'SIGTERM'];
const main = async () => {

  const logger = getLogger("mock-enclave")

  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider: provider });

  const db = level('local');
  const indexer = new Indexer(db);
  const shard = new ShardManager([0]);
  const dispatcher = new Dispatcher();

  await shard.init();

  SHUTDOWN_SIGNALS.forEach(signal => {
    process.once(signal, () => {
      const shutdown = async() => {
        logger.warn("gracefully shutting down  ...")
        await indexer.done();
        await sleep(6000);
        await db.close();
      }

      shutdown().then(() => {
        process.exit(0);
      })
    });
    process.once(signal, () => {
      setTimeout(() => {
        logger.warn(`Could not close resources gracefully after 15s: forcing shutdown`);
        process.exit(1);
      }, 15000)
    })
  });


  try {
    await Storage.getMetadata(db);
  } catch(e) {
    logger.warn(e);
    // local metadata has not been initialized
    await indexer.initialzeLocalDatabase();
  }

  await indexer.fetchShardInfo(api, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');

  // pre-launch validation
  const validedIndexer = await validateIndexer(db, api);
  const validedExecutor = await validateExecutor(db, api);

  if (validedIndexer === -1 || validedExecutor === - 1) {
    logger.error("pre-launch validation failed. Local db corruption. Gotta reset local db. Exiting ...");
    process.exit(1);
  }

  logger.info(`highest validated indexed block at #${validedIndexer}`);
  logger.info(`highest validated executed block at #${validedExecutor}`);

  let txBuffer: QueuedTransaction[] = [{
    transaction: null,
    blockNumber: -1,
  }];

  while (true) {
    await indexer.fetchAll(api, validedIndexer);
    await indexer.writeAll();

    let executionSummary = await Storage.getExecutionSummary(db);
    let localMetadata = await Storage.getMetadata(db);

    for (let blockNumber = validedExecutor; 
      blockNumber < localMetadata.high_local_block; blockNumber ++ ) 
    {
      logger.info(`executing block number ${blockNumber}`);
      const block = await Storage.getBlockRecord(db, 0, blockNumber);
      if (blockNumber % 20 === 0) {
        const txs = await shard.maybeRegisterSecretKeeper(api, blockNumber);
        txBuffer.push(
          ...[... (txs ? txs : [])].map(tx => ({
              transaction: tx,
              blockNumber: blockNumber
            })
          )
        );
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

        executionSummary.high_local_execution_block = block.block_number + 1;

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
        txBuffer.push(
          ...[... (await shard.maybeSubmitExecutionReport(api, db, block.block_number, getStateFileHash()))].map(tx => ({
              transaction: tx,
              blockNumber: blockNumber
            })
          )
        );

        txBuffer = await shard.maybeSubmitTxBatch(api, txBuffer, blockNumber);
      }
    }

    txBuffer = await shard.maybeSubmitTxBatch(api, txBuffer);

    await sleep(6000);
  }  
}
main()