import { execSync } from 'child_process';
import { u8aToHex } from '@skyekiwi/util';

import config from './config'

import {
  Calls,buildCalls, Outcomes, parseOutcomes,
} from '@skyekiwi/s-contract';


const callRuntime = (calls: Calls, stateRoot: Uint8Array): Outcomes => {
  const encodedCall = buildCalls(calls);

  const res = execSync(`../target/release/skw-vm-interface \
    --state-file ${config.stateDumpPrefix} \
    --state-root ${u8aToHex(stateRoot)} \
    ${encodedCall.length === 0 ? "" : `--params ${encodedCall}`}`
  ).toString()

  // console.log( res );
  // console.log(parseOutcomes(  JSON.parse(res) ))
  return parseOutcomes(  JSON.parse(res) );
}

export {callRuntime};