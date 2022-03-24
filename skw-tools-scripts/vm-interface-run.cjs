// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const path = require('path');
const { execSync } = require('child_process');

console.log('$ yarn enclave:ci', process.argv.slice(2).join(' '));

const u8aToString = (u8a) => {
    return (new TextDecoder('utf-8')).decode(u8a);
  };
  
const u8aToHex = (bytes) =>
  bytes.reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '');

function generateParams(config) {
  return `\'${JSON.stringify(config)}\'`;
}

function enclaveCI() {

  const stateFile = path.join(__dirname, "../vm-state-dump/interface");

  try {
      execSync("rm ./vm-state-dump/interface__state_dump__ColState");
  } catch(err) {
      // pass
  }

  execSync("cp ./vm-state-dump/empty__state_dump__ColState ./vm-state-dump/interface__state_dump__ColState");
  
  const wasm_file = path.join(__dirname, "../crates/skw-contract-sdk/examples/status-message-collections/res/status_message_collections.wasm");
  let input = [{
      "transaction_action": "create_account",
      "receiver": "bob", 
      "amount": 100,
      "wasm_file": null,
      "method": null, 
      "args": null,
      "to": null,
    }, {
      "transaction_action": "deploy",
      "receiver": "status", 
      "amount": 100,
      "wasm_file": wasm_file.toString(),
      "method": null, 
      "args": null,
      "to": null,
    }, {
      "transaction_action": "call",
      "receiver": "status", 
      "amount": 0,
      "wasm_file": null,
      "method": "set_status", 
      "args": JSON.stringify({message: "hello_from_root"}),
      "to": null,
    }, {
      "transaction_action": "view_method_call",
      "receiver": "status", 
      "amount": null,
      "wasm_file": null,
      "method": "get_status", 
      "args": JSON.stringify({account_id: "root"}),
      "to": null,
  }];

  (() => {
    let output = JSON.parse(execSync(`./target/release/skw-vm-interface \
      --state-file ${stateFile} \
      --state-root ${u8aToHex([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])} \
      --params ${generateParams(input)}`
    ).toString());
    
    console.log("New State Root", u8aToHex(output.state_root));
    for (let res of output.ops) {
      if (res.view_result && res.view_result.length !== 0) {
        console.log("View Result", u8aToString(Buffer.from(res.view_result)));
      }
      if (res.outcome_logs && res.outcome_logs.length !== 0) {
        console.log("Exec Log", u8aToString(Buffer.from(res.outcome_logs)));
      }
      if (res.outcome_status && res.outcome_status.length !== 0) {
        console.log("Exec Status", u8aToString(Buffer.from(res.outcome_status)));
      }
      
      else {
        // console.log(res);
      }
    }
  })();
}

enclaveCI()
