// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
const fs = require("fs");

console.log('$ yarn enclave:ci', process.argv.slice(2).join(' '));

const u8aToString = (u8a) => {
    return (new TextDecoder('utf-8')).decode(u8a);
  };
  
const u8aToHex = (bytes) =>
  bytes.reduce((str, byte) => str + byte.toString(16).padStart(2, '0'), '');


function enclaveCI() {

  const stateFile = path.join(__dirname, "../vm-state-dump/interface");

  try {
      execSync("rm ./vm-state-dump/interface__state_dump__ColState");
  } catch(err) {
      // pass
  }

  execSync("cp ./vm-state-dump/empty__state_dump__ColState ./vm-state-dump/interface__state_dump__ColState");

  (() => {
    // const output_path = path.join(__dirname, "../vm-interface.json");
    // const output_str = fs.readFileSync(output_path);
    // const output = JSON.parse(output_str);
  
    execSync(`./target/release/skw-vm-interface \
      --state-file ${stateFile} \
      --state-root ${u8aToHex([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])} \
      --transaction-action "create_account" \
      --receiver "bob" \
      --amount "100" \
      > vm-interface.json`
    );
  })();


  (() => {
    const output_path = path.join(__dirname, "../vm-interface.json");
    const output_str = fs.readFileSync(output_path);
    const output = JSON.parse(output_str);
    
    const wasm_file = path.join(__dirname, "../crates/skw-contract-sdk/examples/status-message-collections/res/status_message_collections.wasm");

    console.log(output);

    execSync(`./target/release/skw-vm-interface \
      --state-file ${stateFile} \
      --state-root ${u8aToHex(output.new_state_root)} \
      --transaction-action "deploy" \
      --wasm-file ${wasm_file.toString()} \
      --receiver "status" \
      --amount "100" \
      > vm-interface.json`
    );
  })();

  (() => {
    const output_path = path.join(__dirname, "../vm-interface.json");
    const output_str = fs.readFileSync(output_path);
    const output = JSON.parse(output_str);

    console.log(output);

    execSync(`./target/release/skw-vm-interface \
      --state-file ${stateFile} \
      --state-root ${u8aToHex(output.new_state_root)} \
      --transaction-action "call" \
      --receiver "status" \
      --method "set_status" \
      --args \'${JSON.stringify({message: "hello_from_root"})}\' \
      --amount "0" \
      > vm-interface.json`
    );
  })();

  (() => {
    const output_path = path.join(__dirname, "../vm-interface.json");
    const output_str = fs.readFileSync(output_path);
    const output = JSON.parse(output_str);

    console.log(output);

    execSync(`./target/release/skw-vm-interface \
      --state-file ${stateFile} \
      --state-root ${u8aToHex(output.new_state_root)} \
      --transaction-action "view_method_call" \
      --receiver "status" \
      --method "get_status" \
      --args \'${JSON.stringify({account_id: "root"})}\' \
      > vm-interface.json`
    );
  })();

  (() => {
    const output_path = path.join(__dirname, "../vm-interface.json");
    const output_str = fs.readFileSync(output_path);
    const output = JSON.parse(output_str);

    console.log(u8aToString(Buffer.from(output.view_result)));
    console.log(output);
  })()
  

}

enclaveCI()
