// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from 'fs';
import path from 'path';

const { execute } = require('./execSync');

console.log('$ yarn test', process.argv.slice(2).join(' '));

function build() {
  const srcPath = path.join(__dirname, '../src');

  // compile the runner
  execute(`cd ${srcPath}/near-vm-logic && cargo test --release`);
  execute(`cd ${srcPath}/near-vm-runner && cargo test --release`);
  execute(`cd ${srcPath}/near-vm-errors && cargo test --release`);
}

build();
