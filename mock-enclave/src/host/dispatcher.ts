// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path';
import Level from 'level';
import fs from 'fs';

import {IPFS} from '@skyekiwi/ipfs'
import {  Calls, Call, Outcomes } from '@skyekiwi/s-contract';
import { hexToU8a } from '@polkadot/util';

import { Storage } from './storage';
import {Indexer} from './indexer'

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Dispatcher {

  public async isDispatchable(db: Level.LevelDB, blockNumber: number): Promise<boolean> {
    const summary = await Storage.getExecutionSummary(db);
    return summary.high_local_execution_block < blockNumber;
  }

  public async dispatchNewContract(
    indexer: Indexer, db: Level.LevelDB, 
    contractName: string, stateRoot: Uint8Array,
    executor: (calls: Calls, stateRoot: Uint8Array) => Outcomes
  ): Promise<Uint8Array> {

    const wasmPath = path.join(__dirname, "../../wasm/", contractName + '.wasm');
    const contract = await Storage.getContractRecord(db, contractName);
    const ipfs = new IPFS();

    const content = await ipfs.cat(contract.wasm_blob);

    // can this be exploited?
    fs.writeFileSync(wasmPath, hexToU8a(content));

    // now we deal with the calls 
    let rawOps = contract.deployment_call.ops;

    // push the deployment call to the ops
    let verifiedOps: Call[] = [];
    verifiedOps.push(new Call({
      origin: 'deployer',
      origin_public_key: new Uint8Array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
      encrypted_egress: false,

      transaction_action: 'deploy',
      receiver: contractName,
      amount: 1,
      wasm_blob_path: wasmPath.toString(),
      method: null,
      args: null,
      to: null
    }));

    // validate deployment calls
    for (const op of rawOps) {
      if (op.transaction_action === "deploy") {
        // needs to be removed from the rawOps
        console.log("No deployment calls, will be rejected by runtime either way");
        continue;
      }

      // some other filters
      verifiedOps.push(op)
    }

    const c = new Calls({ "ops": verifiedOps});

    const o = executor(c, stateRoot)
    indexer.writeOutcomes(0, contract.deployment_call_index, o);
    return o.state_root;
  }

  public async dispatchCalls(
    indexer: Indexer, db: Level.LevelDB, 
    callsIndex: number, stateRoot: Uint8Array,
    executor: (calls: Calls, stateRoot: Uint8Array) => Outcomes
  ): Promise<Uint8Array> {
    const c = await Storage.getCallsRecord(db, 0, callsIndex)
    // const ops = c.ops.filter(op => op.transaction_action !== "deploy");
    const ops = c.ops;
    ops.map(it => {
      it.origin = it.origin.toLowerCase();
      it.receiver = it.receiver.toLowerCase();
    })
    const o = executor(new Calls({ ops: ops }), stateRoot);

    indexer.writeOutcomes(0, callsIndex, o);

    return o.state_root;
  }
}
