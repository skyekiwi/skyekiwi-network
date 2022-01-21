// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from 'fs';
import { fromByteArray, toByteArray } from 'base64-js';
import { u8aToString } from '@skyekiwi/util';

const { execute } = require('./execSync');

console.log('$ yarn vm', process.argv.slice(2).join(' '));

function compile() {
  // compile the runner
  execute('cd .. && cargo build --release')
}


const defaultContext = {
  current_account_id: 'contract.sk',
  signer_account_id: 'system.sk',
  signer_account_pk: '15T',
  predecessor_account_id: 'system.sk',
  input: '',
  block_number: 1,
  block_timestamp: '1586796191203000000',
  epoch_height: 1,
  account_balance: '10000000000000000000000000',
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

function runVM({
  methodName = "",
  stateInput = "{}",
  input = "",
  origin = "system.sk",
  wasmFile = "./wasm/status_message.wasm",
  profiling = false
}) {
  const runnerPath = "../target/release/skw-vm-engine-cli";
  execute(`${runnerPath} \
    --context '${injectOrigin(origin)}' \
    --wasm-file ${wasmFile} \
    --method-name ${methodName} \
    ${input ? "--input \'" + input + "\'" : "" } \
    ${stateInput ? "--state \'" + stateInput + "\'" : "" } \
    ${profiling ? "--timings" : ""} \
    > result.json`)

  // parse the output 
  const contentRaw = fs.readFileSync('result.json');
  const content = JSON.parse(contentRaw.toString());
  const stateB64 = JSON.parse(content.state);
  let state: { [key: string]: string } = {}

  for (const key in stateB64) {
    const k = u8aToString(toByteArray(key))
    const v = u8aToString(toByteArray(stateB64[key]))
    state[k] = v;
  }

  console.log()
  console.log("-------EXEC RESULT BEGINS-------");
  console.log("VM Output");
  console.log(content);
  console.log("Current State");
  console.log(state);

  try {
    console.log("Return Value", u8aToString(Uint8Array.from(JSON.parse(content.outcome))));
  } catch (err) {
    // pass - in case of the outcome is 'None'
  }

  console.log("------- EXEC RESULT ENDS -------");
  console.log()

  return stateB64;
}

// compile()

let state = {}

state = runVM({
  methodName: 'set_status',
  input: JSON.stringify({'message': 'jkj'}),
  origin: "system.sk",
  stateInput: '{}',
})

state = runVM({
  methodName: 'set_status',
  input: JSON.stringify({ 'message': 'bob_hello' }),
  origin: "bob.sk",
  stateInput: JSON.stringify(state),
})

state = runVM({
  methodName: 'get_status',
  input: JSON.stringify({ 'account_id': 'bob.sk' }),
  stateInput: JSON.stringify(state),
})

// state = runVM({
//   methodName: 'get_status',
//   input: JSON.stringify({ 'account_id': 'system.sk' }),
//   stateInput: JSON.stringify(state),
// })
// state = runVM({
//   contextFile: './context/bob.json',
//   methodName: 'set_greeting',
//   input: '{"message": "bob_hello"}',
//   stateInput: JSON.stringify(state),
// })

// state = runVM({
//   contextFile: './context/zs.json',
//   methodName: 'set_greeting',
//   input: '{"message": "zs_hello"}',
//   stateInput: JSON.stringify(state),
// })

// state = runVM({
//   methodName: 'get_greeting',
//   input: '{"account_id": "bob.sk"}',
//   stateInput: JSON.stringify(state),
// })

// console.log(state);
