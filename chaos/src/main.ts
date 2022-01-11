// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import {execSync} from 'child_process';
import path from 'path'

import { getLogger } from '@skyekiwi/util';
import { Keyring } from '@polkadot/keyring'
import { waitReady } from '@polkadot/wasm-crypto'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { sendTx } from './util'

function execute(cmd: string) {
  try {
    execSync(cmd, { stdio: 'inherit' });
  } catch (error) {
    process.exit(-1);
  }
};

// whether do we want to fund all accounts - enable for first run OR blockchain reset 
const fundAccounts = true;

const main = async () => {

  await waitReady();
  const rootKeypair = (new Keyring({ type: 'sr25519' })).addFromUri("//Alice");

  const provider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({ provider: provider });

  // deploy one contract - in case of blockchain reset 
  const registerContract = api.tx.sContract.registerContract(
    'QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N',
    '38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f',
    '1111111111222222222211111111112222222222'
  );
  await sendTx(registerContract, rootKeypair, getLogger('deployContract'));

  if (fundAccounts) {
    // fund all accounts
    for (let i = 1; i <= 10; i++) {
      const keyring = (new Keyring({ type: 'sr25519' })).addFromUri(`//${i}`);
      // fund the account with enough gas for 20 push calls
      await sendTx(api.tx.balances.transfer(keyring.address, 155_000_142 * 20), rootKeypair, getLogger(`fund account //${i}`));
    } 
  }
  
  // spawn all workers

  const pm2Path = path.join(__dirname, '../node_modules/.bin/pm2')
  const tsnodePath = path.join(__dirname, '../node_modules/.bin/ts-node')
  const indexPath = path.join(__dirname, './index.ts')
  const logBasePath = path.join(__dirname, './logs')

  // remove all previous log files
  execute(`rm ${logBasePath}/*.log`)

  // each account will make 10 random push calls
  const callCounts = 10;

  for (let i = 1; i <= 10; i++) {
    execute(`${pm2Path} start "${tsnodePath} ${indexPath} ${i} ${callCounts}" --log ${logBasePath}/${i}.log`);
  }  
}

main();
