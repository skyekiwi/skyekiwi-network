// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { waitReady } from '@polkadot/wasm-crypto'
import {genesis} from './genesis'

const main = async () => {

  await waitReady();
  
  // spawn all workers
  const pm2Path = path.join(__dirname, '../node_modules/.bin/pm2')
  const tsnodePath = path.join(__dirname, '../node_modules/.bin/ts-node')
  const indexPath = path.join(__dirname, './index.ts')
  const logBasePath = path.join(__dirname, './logs')

  // 1. launch the blockchain
  // execSync(`${pm2Path} start "${blockchainNode} --tmp --dev"`);

  // 2. genesis config & deploy one contract
  if (process.argv[2] === 'genesis') await genesis()

  // try {
  //   // remove all previous log files
  //   execSync(`rm ${logBasePath}/*.log`)
  // } catch(e) {
  //   // pass
  // }

  // // each account will make 10 random push calls
  // const callCounts = 100;

  // for (let i = 1; i <= 20; i++) {
  //   execSync(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  // }
}

main();
