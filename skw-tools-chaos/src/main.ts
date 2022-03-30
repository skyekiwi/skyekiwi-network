// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { waitReady } from '@polkadot/wasm-crypto'
import {genesis} from './genesis'
import { deployContract } from './deploy';

// whether do we want to fund all accounts - enable for first run OR blockchain reset 
const g = true;
const sleep = (ms: number) => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

const main = async () => {

  await waitReady();
  
  // spawn all workers
  const pm2Path = path.join(__dirname, '../node_modules/.bin/pm2')
  const blockchainNode = path.join(__dirname, '../../target/release/skyekiwi-node')
  const mockEnclaveDB = path.join(__dirname, '../../mock-enclave/local')
  const mockEnclaveFolder = path.join(__dirname, '../../mock-enclave')

  const stateDump = path.join(__dirname, '../../vm-state-dump/interface__state_dump__ColState')
  const emptyStateDump = path.join(__dirname, '../../vm-state-dump/empty__state_dump__ColState')

  const tsnodePath = path.join(__dirname, '../node_modules/.bin/ts-node')
  const indexPath = path.join(__dirname, './index.ts')
  const logBasePath = path.join(__dirname, './logs')

  // 1. launch the blockchain
  // execSync(`${pm2Path} start "${blockchainNode} --tmp --dev"`);

  // 2. genesis config & deploy one contract
  if (g) await genesis()
  await deployContract()

  try {
    // remove all previous log files
    execSync(`rm ${logBasePath}/*.log`)
    execSync(`rm -rf ${mockEnclaveDB}`);
    execSync(`rm ${stateDump}`);
    execSync(`cp ${emptyStateDump} ${stateDump}`);
  } catch(e) {
    // pass
  }

  // each account will make 10 random push calls
  const callCounts = 1000;

  for (let i = 1; i <= 20; i++) {
    execSync(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  }
}

main();
