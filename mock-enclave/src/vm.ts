// import path from 'path';
import { execSync } from 'child_process';
import { u8aToHex } from '@skyekiwi/util';


import config from './config'

// import BN from 'bn.js';
// import path from 'path'

import {
  Calls,buildCalls, parseOutcomes, Outcomes,
} from './host/borsh';


const callRuntime = (calls: Calls, stateRoot: Uint8Array, resetState = false): Outcomes => {
  if (resetState) {
    try {
      execSync(`rm ${config.currentStateFile}`);
      execSync(`cp ${config.genesisStateFile} ${config.currentStateFile}`);
    } catch(err) { }
  }

  const encodedCall = buildCalls(calls);
  return parseOutcomes(  JSON.parse(execSync(`../target/release/skw-vm-interface \
      --state-file ${config.stateDumpPrefix} \
      --state-root ${u8aToHex(stateRoot)} \
      ${encodedCall.length === 0 ? "" : `--params ${encodedCall}`}`
    ).toString()));
}

// const callStatus = (resetState: boolean) => {
//   const status_wasm_file = path.join(__dirname, "../wasm/status_message_collections.wasm");
//   const calls = new Calls({
//     ops: [ 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "deploy",
//         "receiver": "status", 
//         "amount": new BN(0x100, 16),
//         "wasm_blob_path": status_wasm_file.toString(),
//         "method": null, 
//         "args": null,
//         "to": null,
//       }), 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "call",
//         "receiver": "status", 
//         "amount": null,
//         "wasm_blob_path": null,
//         "method": "set_status", 
//         "args": JSON.stringify({message: "HELLLOOOOOOOOOOO"}),
//         "to": null,
//       }), 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "view_method_call",
//         "receiver": "status", 
//         "amount": null,
//         "wasm_blob_path": null,
//         "method": "get_status", 
//         "args": JSON.stringify({account_id: "root"}),
//         "to": null,
//       }),
//     ]
//   });
//   const o = callRuntime(calls, new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]), resetState);
//   return o.state_root;
// }


// const callCrossContract = (state_root: Uint8Array, resetState: boolean) => {
//   const contract_wasm_file = path.join(__dirname, "../wasm/cross_contract_high_level.wasm");
  
//   const calls = new Calls({
//     ops: [ 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "deploy",
//         "receiver": "contract", 
//         "amount": new BN(0x100, 16),
//         "wasm_blob_path": contract_wasm_file.toString(),
//         "method": null, 
//         "args": null,
//         "to": null,
//       }), 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "call",
//         "receiver": "contract", 
//         "amount": null,
//         "wasm_blob_path": null,
//         "method": "deploy_status_message", 
//         "args": JSON.stringify({account_id: "status", amount: new BN(0x100, 16)}),
//         "to": null,
//       }), 
//       new Call({
//         "origin": "root",
//         "origin_public_key": new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
    
//         "encrypted_egress": false,

//         "transaction_action": "call",
//         "receiver": "contract", 
//         "amount": null,
//         "wasm_blob_path": null,
//         "method": "complex_call", 
//         "args": JSON.stringify({account_id: "status", message: "something"}),
//         "to": null,
//       }),
//     ]
//   });
//   return callRuntime(calls, state_root, resetState);
// }

// const root = callStatus(true);
// console.log(callCrossContract(root, false));

export {callRuntime};