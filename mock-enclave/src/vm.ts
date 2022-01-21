import fs from 'fs';
import { fromByteArray, toByteArray } from 'base64-js';
import { u8aToString } from '@skyekiwi/util';
import configuration from './config'

const { execute } = require('../scripts/execSync');

export function preRun() {
  execute('cd src/near-vm-runner-standalone && cargo build --release')
}

const defaultContext = {
  current_account_id: 'contract.sk',
  signer_account_id: 'system.sk',
  signer_account_pk: '15T',
  predecessor_account_id: 'system.sk',
  input: '',
  block_index: 1,
  block_timestamp: '1586796191203000000',
  epoch_height: 1,
  account_balance: '10000000000000000000000000',
  account_locked_balance: '0',
  storage_usage: 100,
  attached_deposit: '0',
  prepaid_gas: 1000000000000000000,
  random_seed: '15T',
  view_config: null,
  output_data_receivers: []
}

const injectOrigin = (origin: string) => {
  let thisContext = defaultContext;
  thisContext['signer_account_id'] = origin;
  return JSON.stringify(thisContext);
}

export function runVM({
  methodName = "",
  stateInput = "{}",
  input = "",
  wasmFile = "./wasm/greeting.wasm",
  origin = "system.sk",
  profiling = false,
  contractId = "0x000000"
}): string {
  const outputPath = `${configuration.localStoragePath}${contractId}.json`;
  const runnerPath = "./src/near-vm-runner-standalone/target/release/near-vm-runner-standalone";
  execute(`${runnerPath} --context '${injectOrigin(origin)}' --wasm-file '${wasmFile}' --method-name '${methodName}' --input ${input} --state ${stateInput} ${profiling ? "--timings" : ""} > ${outputPath}`)
  
  // parse the output 
  const contentRaw = fs.readFileSync(outputPath);
  const content = JSON.parse(contentRaw.toString());
  const stateB64 = JSON.parse(content.state);
  let state: {[key: string]: string} = {}
  
  for (const key in stateB64) {
    const k = u8aToString(toByteArray(key))
    const v = u8aToString(toByteArray(stateB64[key]))
    state[k] = v;
  }
  console.log(state)
  return stateB64;
}
