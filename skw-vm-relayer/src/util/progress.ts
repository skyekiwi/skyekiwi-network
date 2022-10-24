export const progressText: {
    [key: string]: (arg0?: number, args?: number) => string
  } = {
    DISPATCHER_EXECUTION_SKIPPING: () => '💤 No Calls in Buffer. Sleeping for 6 Seconds. ',
    DISPATCHER_EXECUTION_CALL_VALIDATED: (callIndex: number) => `👀 call validated for ${callIndex}, sending to executor`,
    DISPATCHER_EXECUTION_BUILDING_PAYLOAD: (blockNumber: number) => ` 💅 Building Payload for BlockNumber #${blockNumber}`,
    DISPATCHER_EXECUTION_DISPATCHING: (blockNumber: number) => `🛠 dispatching BlockNumber #${blockNumber}`,
    DISPATCHER_EXECUTION_DONE: (blockNumber: number) => `🛠 Execution Finished For BlockNumber #${blockNumber}, sending into submission buffer`,
    
    INDEXER_FETCH_START: (blockNumber: number) => `💁 highest local block at ${blockNumber}`,
    INDEXER_FETCH_FETCHING: (blockNumber: number) => `⬇️ fetching all info from block# ${blockNumber}`,
    INDEXER_FETCH_FETCH_BLOCK: (blockNumber: number, callLength: number) => `📦 block import complete at BlockNumber # ${blockNumber}, imported ${callLength} calls`,
    INDEXER_FETCH_ALLDONE: (blockNumber: number) => `✅ all catchuped ... for now at block# ${blockNumber}`,
    INDEXER_FETCH_WRITE_BUFFED: () => `💯 writting some buffered blocks to local DB`,

    SUBMITTER_SKIP_SUBMISSION: () => `😴 No execution in buffer, awaiting`,
    SUBMITTER_BLOCK_GENERATED: (blockNumber: number) => `📩 Outcome payload buffered for BlockNumber #${blockNumber}`,
    SUBMITTER_SUBMITTED: () => `🌞 Transactions Submitted. All up to date`,
  };
  