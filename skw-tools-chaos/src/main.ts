// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { waitReady } from '@polkadot/wasm-crypto'
import {genesis} from './genesis'
import { deployContract } from './deploy';

// whether do we want to fund all accounts - enable for first run OR blockchain reset 
const g = true;

const main = async () => {

  await waitReady();

  if (g) await genesis()
  await deployContract()

  // spawn all workers

  const pm2Path = path.join(__dirname, '../node_modules/.bin/pm2')
  const tsnodePath = path.join(__dirname, '../node_modules/.bin/ts-node')
  const indexPath = path.join(__dirname, './index.ts')
  const logBasePath = path.join(__dirname, './logs')

  try {
    // remove all previous log files
    execSync(`rm ${logBasePath}/*.log`)
  } catch(e) {
    // pass
  }
  // each account will make 10 random push calls
  const callCounts = 1000;

  for (let i = 1; i <= 100; i++) {
    execSync(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  }  
}

main();
