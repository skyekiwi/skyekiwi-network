// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import {genesis} from './genesis'


// whether do we want to fund all accounts - enable for first run OR blockchain reset 
const g = true;

const main = async () => {

  await waitReady();
  const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri("//Alice");

  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider: provider });

  if (g) genesis()
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
  const callCounts = 10;

  for (let i = 1; i <= 10; i++) {
    execSync(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  }  
}

main();
