// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn enclave:ci', process.argv.slice(2).join(' '));

function generateChainSpec() {

  const node = path.join(__dirname, "../target/release/skyekiwi-node");
  const resFolder = path.join(__dirname, "../crates/skw-blockchain-node/res");
  execSync(`${node} build-spec \
    --chain=skw_alpha \
    --raw \
    --disable-default-bootnode > ${resFolder}/alphaRaw.json`);
}

generateChainSpec()
