import { execSync } from 'child_process';
import { u8aToHex } from '@skyekiwi/util';

import config from './config'

const callRuntime = (encodedCalls: string, stateRoot: Uint8Array): string => {
  return JSON.parse(execSync(`../target/release/skw-vm-interface \
    --state-file ${config.stateDumpPrefix} \
    --state-root ${u8aToHex(stateRoot)} \
    ${encodedCalls.length === 0 ? "" : `--params ${encodedCalls}`} \
    --dump-state`).toString())
}

const recoverState = (originFile: string, outputFile: string, rawPatches: Uint8Array[]) => {

  const padSize = (size: number): Uint8Array => {
    const res = new Uint8Array(4);
  
    res[0] = size & 0xff;
    res[1] = (size >> 8) & 0xff;
    res[2] = (size >> 16) & 0xff;
    res[3] = (size >> 24) & 0xff;
  
    return res;
  };

  let patch = new Uint8Array(0);
  for (const rawPatch of rawPatches) {
    patch = new Uint8Array([...patch, ...padSize(rawPatch.length), ...rawPatch]);
  }

  execSync(`../target/release/skw-vm-patch \
    --state-file ${originFile} \
    --state-patch ${u8aToHex(patch)} \
    --output ${outputFile}`
  )
}

export {callRuntime, recoverState};