import { ApiPromise, WsProvider } from '@polkadot/api';
import level from 'level'

import {baseDecode, baseEncode} from 'borsh';

import {Calls, parseRawOutcomes, Outcomes, BlockSummary, buildCalls, ExecutionSummary} from '@skyekiwi/s-contract/borsh';
import { getLogger} from '@skyekiwi/util';

import { Indexer } from './host/indexer';
import { Storage } from './host/storage'
import { ShardManager } from './host/shard';

import { Dispatcher } from './host/dispatcher';
import { callRuntime  } from './vm';
import { validateIndexer, validateExecutor } from './validate';

import { DBOps, QueuedTransaction } from './host/types';

require("dotenv").config()

const padSize = (size: number): Uint8Array => {
  const res = new Uint8Array(4);

  res[3] = size & 0xff;
  res[2] = (size >> 8) & 0xff;
  res[1] = (size >> 16) & 0xff;
  res[0] = (size >> 24) & 0xff;

  return res;
};

const unpadSize = (size: Uint8Array): number => {
  return size[3] | (size[2] << 8) | (size[1] << 16) | (size[0] << 24);
};

const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

const processCalls = async (
  calls: {[key: number]: Calls},
  blockNumber: number,
  lastStateRoot: Uint8Array
): Promise<[DBOps[], Uint8Array]> => {

  if (Object.keys(calls).length === 0) {
    return [[], new Uint8Array(0)];
  }

  let lastBlockSummary = new BlockSummary({
    block_number: blockNumber - 1,
    block_state_root: lastStateRoot,
    contract_state_patch_from_previous_block: new Uint8Array(0),
    call_state_patch_from_previous_block: new Uint8Array(0),
  });

  let dbOps: DBOps[] = [];
  let blockSummary = new BlockSummary({
    block_number: blockNumber,
    block_state_root: new Uint8Array(0),
    contract_state_patch_from_previous_block: new Uint8Array(0),
    call_state_patch_from_previous_block: new Uint8Array(0),
  })

  // Execute Calls
  let callRaw = new Uint8Array(0);
  for (const callIndex in calls) {
    const call = calls[callIndex];
    const encodedCall = baseDecode(buildCalls(call));
    callRaw = new Uint8Array([
      ...callRaw,
      ...padSize(encodedCall.length + 4),
      ...padSize(Number(callIndex)),
      ...encodedCall,
    ]);
  }

  // 3. send for execution
  const callOutcome = callRuntime(baseEncode(callRaw), lastBlockSummary.block_state_root)

  // 4. parse outcomes
  const decodedCallOutcome = baseDecode(callOutcome);
  let callOutcomeOffset = 0;
  
  while (callOutcomeOffset < callOutcome.length) {
    const outcomeSize = unpadSize(decodedCallOutcome.slice(callOutcomeOffset, callOutcomeOffset + 4));
    if (outcomeSize === 0) {
      break;
    }
    const callIndex = unpadSize(decodedCallOutcome.slice(callOutcomeOffset + 4, callOutcomeOffset + 8));
    const rawOutcome = parseRawOutcomes(baseEncode(decodedCallOutcome.slice(callOutcomeOffset + 8, callOutcomeOffset + 4 + outcomeSize)));

    callOutcomeOffset += 4 + outcomeSize;
    lastBlockSummary.block_state_root = rawOutcome.state_root;
    blockSummary.block_state_root = lastBlockSummary.block_state_root;
    blockSummary.call_state_patch_from_previous_block = rawOutcome.state_patch;
    
    const outcome = new Outcomes({
      ops: rawOutcome.ops,
      state_root: rawOutcome.state_root,
    })
    dbOps.push(Storage.writeCallOutcome(0, callIndex, outcome));
  }

  dbOps.push(Storage.writeBlockSummary(0, blockNumber, blockSummary));
  dbOps.push(Storage.writeExecutionSummary(new ExecutionSummary({
    high_local_execution_block: blockNumber,
  })))
  return [dbOps, lastBlockSummary.block_state_root]
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

  let lightSwitch = true;

  SHUTDOWN_SIGNALS.forEach(signal => {
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


  try {
    await Storage.getMetadata(db);
  } catch(e) {
    logger.warn(e);
    // local metadata has not been initialized
    await indexer.initialzeLocalDatabase();
  }

  await indexer.fetchShardInfo(api, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');

  // pre-launch validation
  let validedIndexer = await validateIndexer(db, api);
  let validedExecutor = await validateExecutor(db, api);

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

  while (true && lightSwitch) {

    let localMetadata = await Storage.getMetadata(db);
    let executionSummary = await Storage.getExecutionSummary(db);

    await indexer.fetchAll(api, localMetadata.high_local_block);
    await indexer.writeAll();
    localMetadata = await Storage.getMetadata(db);
    executionSummary = await Storage.getExecutionSummary(db);


    for (let blockNumber = executionSummary.high_local_execution_block; 
      blockNumber < localMetadata.high_local_block; blockNumber ++ ) 
    {
      try {
        await Storage.getBlockSummary(db, 0, blockNumber)
      } catch(e) {
        // the block has not been executed
        // console.log(blockNumber, localMetadata.high_local_block)
        logger.info(`🙌 sending block number ${blockNumber} for execution`);
        const block = await Storage.getBlockRecord(db, 0, blockNumber);
      
        let callsOfBlock: {[key: number]: Calls} = {}
        // check whether to register the secret keeper & push to the tx buffer
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

        // the main process of executing
        if (dispatcher.isDispatchable(db, blockNumber)) {

          let deploymentCalls: number[] = [];
          if (block.contracts && block.contracts.length > 0) {
            for (const contractName of block.contracts) {
              const [newCalls, deploymentCallIndex] = await dispatcher.dispatchNewContract(db, contractName);
              deploymentCalls.push(deploymentCallIndex);
              callsOfBlock[deploymentCallIndex] = newCalls;
            }
          }

          if (block.calls && block.calls.length > 0) {
            for (let call of block.calls) {
              if (deploymentCalls.filter(i => i === call).length === 0) {
                const newCalls = await dispatcher.dispatchCalls(db, call)
                callsOfBlock[call] = newCalls;
              }
            }
          }

          if (Object.keys(callsOfBlock).length > 0) {
            const [dbOps, newStateRoot] = await processCalls(
              callsOfBlock, 
              blockNumber,
              localMetadata.latest_state_root,
            );
            localMetadata.latest_state_root = newStateRoot; 

            // console.log(localMetadata)

            dbOps.push(Storage.writeMetadata(localMetadata));
            await Storage.writeAll(db, dbOps);
          }
        }

        const conf = (await api.query.parentchain.confirmation(0, block.block_number)).toJSON();
        if (
            // no reports has been submitted yet || confirmation is below the threshold?
            (conf === null) && (
              (block.calls && block.calls.length !== 0) || 
              (block.contracts && block.contracts.length !== 0)
            ) && dispatcher.isDispatchable(db, block.block_number)
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