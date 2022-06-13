// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { waitReady } from '@polkadot/wasm-crypto'
import {genesis} from './genesis'

import { buildCalls, parseCalls } from '@skyekiwi/s-contract';
import { hexToU8a } from '@skyekiwi/util';
import {baseEncode} from 'borsh';


const main = async () => {

  // const call = "010000006d6f646c73636f6e7472616300000000000000000000000000000000000000008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48000001e80300000000000000000000";
  // const rawCall = hexToU8a(call);

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

  try {
    // remove all previous log files
    execSync(`rm ${logBasePath}/*.log`)
  } catch(e) {
    // pass
  }

  // // each account will make 10 random push calls
  // const callCounts = 10;

  // for (let i = 1; i <= 20; i++) {
  //   execSync(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  // }
}

main();
