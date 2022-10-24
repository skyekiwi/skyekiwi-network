import { ApiPromise } from '@polkadot/api';
import Level, { LevelDB } from 'level'
import { LocalMetadata, ExecutionSummary, buildCalls, buildOutcomes, baseEncode  } from '@skyekiwi/s-contract/borsh';
import { hexToU8a, u8aToString} from '@skyekiwi/util';
import { Storage } from './host/storage'
import { CallRecord } from './host/types';
import { getLogger } from './util';

const overwriteLocalMetadata = async (db: Level.LevelDB, localMetadata: LocalMetadata) => {
  await Storage.writeAll(db, [
    Storage.writeMetadata(localMetadata),
  ]);
}

const overwriteExecutionSummary = async (db: Level.LevelDB, executionSummary: ExecutionSummary) => {
  await Storage.writeAll(db, [
    Storage.writeExecutionSummary(executionSummary),
  ]);
}

const validateIndexer = async (db: LevelDB, api: ApiPromise, start?: number, end?: number): Promise<number> => {
  const logger = getLogger("validation.indexer")

  try {
    const metadata = await Storage.getMetadata(db)
    start = start ? start : 1;
    end = end ? end : metadata.high_local_block;

    for (let i = start; i <= end; i ++) {
      try {
        const localBlock = await Storage.getBlockRecord(db, 0, i);

        // have calls
        if (localBlock && localBlock.calls && localBlock.calls.length) {

          logger.info(`validating block #${i}`);

          const remoteCallHistroy = (await api.query.sContract.callHistory(0, i)).toJSON() as unknown as number[];
          if (!remoteCallHistroy) {
            logger.error(`BlockNumber#${i} remote call history is null, while local call is not, exiting`);
            
            metadata.high_local_block = i - 1;
            await overwriteLocalMetadata(db, metadata);
            return i - 1;
          }
          if (remoteCallHistroy.length !== localBlock.calls.length) {
            logger.error(`BlockNumber#${i} remote call history length is ${remoteCallHistroy.length}, while local call is ${localBlock.calls.length}, exiting`);
            
            metadata.high_local_block = i - 1;
            await overwriteLocalMetadata(db, metadata);
            return i - 1;
          }

          const remoteCallIdSum = remoteCallHistroy.reduce((acc, cur) => acc + cur, 0);
          const localCallIdSum = localBlock.calls.reduce((acc, cur) => acc + cur, 0);

          // 1. validate call history
          if (remoteCallIdSum !== localCallIdSum) {
            logger.error(`BlockNumber#${i} remote call id sum is ${remoteCallIdSum}, while local call id sum is ${localCallIdSum}, exiting`);
            
            metadata.high_local_block = i - 1;
            await overwriteLocalMetadata(db, metadata);
            return i - 1;
          }

          for (const callId of remoteCallHistroy) {
            // 2. Validate call record optional? it is very hard for this part to go wrong. 
            const remoteCallRecord = (await api.query.sContract.callRecord(callId)).toJSON() as CallRecord;
            const remoteCallContent = hexToU8a(remoteCallRecord[0].substring(2))

            try {
              const localCallRecord = await Storage.getCallsRecord(db, 0, callId);
              if ( baseEncode(remoteCallContent) !== buildCalls(localCallRecord)) {
                logger.error(`local callRecord at ${callId} does not match remote callRecord`);
                
                metadata.high_local_block = i - 1;
                await overwriteLocalMetadata(db, metadata);
                return i - 1;
              }
            } catch(e) {
              logger.error(`BlockNumber#${i} callId#${callId} failed to fetch local db records`);
              
              metadata.high_local_block = i - 1;
              await overwriteLocalMetadata(db, metadata);
              return i - 1;
            }
          }
        }
      } catch(e) {
        logger.error(`Block#${i} local record pulling fault`);
        logger.error(e);
        
        metadata.high_local_block = i - 1;
        await overwriteLocalMetadata(db, metadata);
        return i - 1;
      }
    }

    return end;

  } catch(e)  {
    logger.error(`!Fatal Error! failed to fetch metadata`);
    return -1;
  }
}


const validateExecutor = async (db: LevelDB, api: ApiPromise, start?: number, end?: number): Promise<number> => {
  // given that we trust all indexer output
  
  const logger = getLogger("validation.executor")
  try {
    const metadata = await Storage.getMetadata(db)

    const executionSummary = await Storage.getExecutionSummary(db);
    start = start ? start : 1;
    end = end ? end : executionSummary.high_local_execution_block;


    if (executionSummary.high_local_execution_block > metadata.high_local_block) {
      logger.error("!Fatal Error!  high_local_execution_block > high_local_block, resetting.");

      executionSummary.high_local_execution_block = 1;
      await overwriteExecutionSummary(db, executionSummary);
      return -1;
    }

    for (let i = start; i <= end; i ++) {
      try {
        const localBlock = await Storage.getBlockRecord(db, 0, i);
        
        try {
          const blockSummary = await Storage.getBlockSummary(db, 0, i);
          
          // because the pulling is success - the block has calls
          if (!blockSummary.block_state_root ||
            !blockSummary.call_state_patch_from_previous_block || 
            !blockSummary.contract_state_patch_from_previous_block) {
            
              // should init patching 
            executionSummary.high_local_execution_block = i - 1;
            await overwriteExecutionSummary(db, executionSummary);
            return i - 1;
          }

          // have calls
          if (localBlock && localBlock.calls && localBlock.calls.length) {

            logger.info(`validating block #${i}`);

            for (const callId of localBlock.calls) {
              let remoteCallOutcomeContent;
              const remoteCallOutcome = (await api.query.parentchain.outcome(callId)).toJSON() as unknown as string;
              
              if (remoteCallOutcome === "0x") {
                remoteCallOutcomeContent = ""
              } else {
                remoteCallOutcomeContent = u8aToString(hexToU8a(remoteCallOutcome.substring(2)));
              }

              try {
                const localCallOutcome = await Storage.getOutcomesRecord(db, 0, callId);
                if (remoteCallOutcomeContent !== buildOutcomes(localCallOutcome)) {

                  console.log(remoteCallOutcomeContent, localCallOutcome);
                  logger.error(`local callOutcome at ${callId} does not match remote callOutcome`);
                  return i - 1;
                }
              } catch(e) {
                logger.error(`BlockNumber#${i} callId#${callId} failed to fetch local db records`);
                return i - 1;
              }
            }
          }
        } catch(err) {
          if (localBlock && localBlock.calls && localBlock.calls.length) {
            logger.error(`pulling block summary error at #${i}`);

            executionSummary.high_local_execution_block = i - 1;
            await overwriteExecutionSummary(db, executionSummary);
            return i - 1;
          } 

          // else pass - empty block
        }
      } catch(e) {
          logger.error(`Block#${i} local record pulling fault`);
          return i - 1;
      }
    }

    return end;

  } catch(e)  {
    logger.error(`!Fatal Error! failed to fetch metadata`);
    return -1;
  }
}

// const validateStatusMessageTests = async (db: LevelDB, api: ApiPromise) => {    
//     const logger = getLogger("validation.state_message_tests")

//     const highCallIndex = (await api.query.sContract.currentCallIndex()).toJSON() as unknown as number;

//     for (let i = 0; i < highCallIndex; i ++) {
//         logger.info(`validating call #${i}`);
//         const call = await Storage.getCallsRecord(db, 0, i);

//         try {
//             if (call.ops.length === 2 && call.ops[1].transaction_action === 'view_method_call') {
//                 // the original pushed in param
//                 const msg: string = JSON.parse(call.ops[0].args)['message'];
                
//                 const outcome = await Storage.getOutcomesRecord(db, 0, i);
//                 const outcomeMsg = JSON.parse(u8aToString( new Uint8Array(outcome.ops[1].view_result) ));
                
//                 // NOTE: local outcome & remote outcome consistency has been validated, therefore, validate local outcome only
//                 if ( msg !== outcomeMsg ) {
//                     logger.error(`call ${i} does not match outcome`);
//                 }
//             }
//         } catch(e) {
//             logger.error(`Call#${i} received ${e}`);
//         }
//     }
// }

export { validateIndexer, validateExecutor }

// const main = async() => {

//     const provider = new WsProvider('ws://localhost:9944');
//     const api = await ApiPromise.create({ provider: provider });
    
//     const db = level('local');

//     await validate(db, api)
//     await validateStatusMessageTests(db, api)
//     await provider.disconnect()
// }

// main()