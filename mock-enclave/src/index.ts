import { ApiPromise, WsProvider } from '@polkadot/api';
import {Calls, Block, Outcomes} from './host/borsh';
import {u8aToString, u8aToHex} from '@skyekiwi/util';
import { Indexer } from './host/indexer';
import {Storage} from './host/storage'
import { ShardManager } from './host/shard';
import { Subscriber } from './host/subscriber';

import { Dispatcher } from './host/dispatcher';
import level from 'level'
import { callRuntime  } from './vm';

require("dotenv").config()

const processCalls = (calls: Calls, stateRoot: Uint8Array) => {
  console.log("Executing", calls)

  if (calls.ops.length === 0) 
    return new Outcomes({
      ops: [],
      state_root: stateRoot,
    });

  const outcomes = callRuntime(calls, stateRoot)

  console.log("New State Root", u8aToHex(outcomes.state_root));
  for (let res of outcomes.ops) {

    console.log("Outcome:", res);
    if (res.view_result && res.view_result.length !== 0) {
      console.log("View Result", u8aToString(Buffer.from(res.view_result)));
    }
    // if (res.outcome_logs && res.outcome_logs.length !== 0) {
    //   console.log("Exec Log", u8aToString(Buffer.from(res.outcome_logs)));
    // }
    if (res.outcome_status && res.outcome_status.length !== 0) {
      console.log("Exec Status", u8aToString(Buffer.from(res.outcome_status)));
    }
    
    else {
      // console.log(res);
    }
  }

  return outcomes;
}

const main = async () => {

  // callStatus(true)

  const subscriber = new Subscriber();
  const indexer = new Indexer();
  const shard = new ShardManager([0]);
  await shard.init();
  const dispatcher = new Dispatcher();

  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider: provider });
  const db = level('local');

  indexer.init(db);
  await indexer.initialzeLocalDatabase();

  await indexer.fetchOnce(api, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
  await indexer.fetchAll(api);
  await indexer.writeAll();
  console.log(`pre-launching indexing done`)

  let executionSummary = await Storage.getExecutionSummary(db);
  let localMetadata = await Storage.getMetadata(db);


  await indexer.runBlocks(
    executionSummary.high_local_execution_block + 1, 
    localMetadata.high_local_block,
    async (block: Block) => {
      if (dispatcher.isDispatchable(db, block.block_number)) {

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

        executionSummary.high_local_execution_block = block.block_number;
        const op = [
          Storage.writeExecutionSummary(executionSummary),
          Storage.writeMetadata(localMetadata),
        ];
        Storage.writeAll(db, op);

        console.log(`execution summary: ${executionSummary.high_local_execution_block}`)
      }

      await indexer.writeAll()

      const conf = (await api.query.parentchain.confirmation(0, block.block_number)).toJSON();
      if (
          conf === null && (
            (block.calls && block.calls.length !== 0) || 
            (block.contracts && block.contracts.length !== 0)
          )
      ) {
        console.log("submitting report for ", block.block_number)
        await shard.maybeSubmitExecutionReport(api, db, block.block_number);
      }
    }
  )
return;
  await subscriber.subscribeNewBlock(api, async (blockNumber: number) => {
    
    console.log(`New block: ${blockNumber}`);
    if (blockNumber % 20 === 0) {
      await shard.maybeRegisterSecretKeeper(api, blockNumber);
      await shard.maybeSubmitExecutionReport(api, db, blockNumber);
    }

    await indexer.fetchAll(api);
    await indexer.writeAll();
    let executionSummary = await Storage.getExecutionSummary(db);
    let localMetadata = await Storage.getMetadata(db);

    await indexer.runBlocks(
      executionSummary.high_local_execution_block, 
      localMetadata.high_local_block,
      async (block: Block) => {
        if (block.contracts && block.contracts.length > 0) {
          for (const contractName of block.contracts) {
            localMetadata.latest_state_root = await dispatcher.dispatchNewContract(indexer, db, contractName, localMetadata.latest_state_root, processCalls);
          }
        }

        if (block.contracts && block.contracts.length > 0) {
          for (const contractName of block.contracts) {
            localMetadata.latest_state_root = await dispatcher.dispatchNewContract(indexer, db, contractName, localMetadata.latest_state_root, processCalls);
          }
        }

        executionSummary.high_local_execution_block = block.block_number;
        const op = [
          Storage.writeExecutionSummary(executionSummary),
          Storage.writeMetadata(localMetadata),
        ];
         Storage.writeAll(db, op);

        console.log(`execution summary: ${executionSummary.high_local_execution_block}`)
        await indexer.writeAll()
      }
    )
  })
}

main()
