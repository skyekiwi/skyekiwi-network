import type { QueuedTransaction } from './host/types';

import { ApiPromise, WsProvider } from '@polkadot/api';
import level from 'level'

import { Calls, ExecutionSummary } from '@skyekiwi/s-contract/borsh';
import { getLogger } from '@skyekiwi/util';
import { sleep } from '@skyekiwi/util/util';

import { Indexer } from './host/indexer';
import { Storage } from './host/storage'
import { ShardManager } from './host/shard';
import { Dispatcher } from './host/dispatcher';

import { validateIndexer, validateExecutor } from './validate';

require("dotenv").config()

const main = async () => {

  const logger = getLogger("mock-enclave")

  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider: provider });

  const db = level('local');
  const indexer = new Indexer(db);

  // define which shard the current machine is running
  const shard = new ShardManager([0]);
  await shard.init();

  let lightSwitch = true;

  // 1. register the shutdown signals for gracefully shutdown
  ['SIGINT', 'SIGTERM'].forEach(signal => {
    process.once(signal, () => {
      lightSwitch = false;
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


  // 2. initialze local storage if it is currently unavalaible 
  try {
    await Storage.getMetadata(db);
  } catch(e) {
    logger.warn(e);
    // local metadata has not been initialized
    await indexer.initialzeLocalDatabase();
  }

  // 3. fetch the current shard info
  await indexer.fetchShardInfo(api, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');

  // 4. pre-launch validation
  let validedIndexer = await validateIndexer(db, api);
  let validedExecutor = await validateExecutor(db, api);

  if (validedIndexer === -1 || validedExecutor === - 1) {
    logger.error("pre-launch validation failed. Local db corruption. Gotta reset local db. Exiting ...");
    process.exit(1);
  }

  logger.info(`highest validated indexed block at #${validedIndexer}`);
  logger.info(`highest validated executed block at #${validedExecutor}`);

  // 5. initialzed the pending transaction queue
  let txBuffer: QueuedTransaction[] = [{
    transaction: null,
    blockNumber: -1,
  }];

  // MAIN LOOP
  while (true && lightSwitch) {

    // 6. sync with the blockchain data
    let localMetadata = await Storage.getMetadata(db);
    await indexer.fetchAll(api, localMetadata.high_local_block);
    await indexer.writeAll();

    localMetadata = await Storage.getMetadata(db);
    let executionSummary = await Storage.getExecutionSummary(db);

    // 7. Main execution loop
    for (let blockNumber = executionSummary.high_local_execution_block; 
      blockNumber < localMetadata.high_local_block; blockNumber ++ ) 
    {
      try {
        // whether the block has been executed?
        await Storage.getBlockSummary(db, 0, blockNumber)
        // Ok -- funny shit happend or we should not be here.
      } catch(e) {

        // the block has not been executed
        const block = await Storage.getBlockRecord(db, 0, blockNumber);
        if (
          !block ||
          (!block.calls || block.calls.length === 0)
        ) {
          logger.info(`🙌 skipping block number ${blockNumber} for execution as it's empty`);
          // empty block

          await Storage.writeAll(db, [Storage.writeExecutionSummary(new ExecutionSummary({
                high_local_execution_block: blockNumber,
                latest_state_root: executionSummary.latest_state_root,
              })
            )]
          )
          continue;
        }

        logger.info(`🙌 sending block number ${blockNumber} for execution`);

        // check whether to register the secret keeper & push to the tx buffer
        if (blockNumber % 20 === 0 || await shard.secretKeeperRegistryExpiredOrMissing(api, blockNumber)) {
          // Extra-8: if not registerd, register ourselves as a secret keeper
          const txs = await shard.maybeRegisterSecretKeeper(api, blockNumber);
          txBuffer.push(
            ...[... (txs ? txs : [])].map(tx => ({
                transaction: tx,
                blockNumber: blockNumber
              })
            )
          );
        }

        // 8. re-pack all calls in this block and send for execution
        let callsOfBlock: {[key: number]: Calls} = {}

        // 9. The main execution loop
        if (Dispatcher.isDispatchable(db, blockNumber)) {

          if (block.calls && block.calls.length > 0) {
            for (let callIndex of block.calls) {
              logger.info(`⬆️ CallIndex ${callIndex} is sent for pre-dispatch processing`);
              const newCalls = await Dispatcher.preDispatchProcessing(api, db, callIndex)
              callsOfBlock[callIndex] = newCalls;
            }
          }

          if (Object.keys(callsOfBlock).length > 0) {
            const [dbOps, newStateRoot] = Dispatcher.dispatchBatch(
              callsOfBlock, 
              executionSummary.latest_state_root,
            );
            executionSummary.latest_state_root = newStateRoot;
            await Storage.writeAll(db, dbOps);
          }
        }
        // 10. check for chain confirmation of the current block and try to decide if we should submit an outcomes
        const conf = (await api.query.parentchain.confirmation(0, block.block_number)).toJSON();
        if (
            // no reports has been submitted yet || confirmation is below the threshold?
            conf === null &&
            (block.calls && block.calls.length !== 0) && 
            Dispatcher.isDispatchable(db, block.block_number)
        ) {
          // console.log("submitting report for ", block.block_number)
          txBuffer.push(
            ...[... (await shard.maybeSubmitExecutionReport(api, db, block.block_number))].map(tx => ({
                transaction: tx,
                blockNumber: blockNumber
              })
            )
          );

          txBuffer = await shard.maybeSubmitTxBatch(api, txBuffer, (await api.query.system.number()).toJSON() as number);
        }
      }
    }
    txBuffer = await shard.maybeSubmitTxBatch(api, txBuffer);
    await indexer.writeAll();

    await sleep(6000);
  }  
}
main()