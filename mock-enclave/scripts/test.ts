// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from 'fs';
import path from 'path';

const { execute } = require('./execSync');

console.log('$ yarn test', process.argv.slice(2).join(' '));

function build() {
  const srcPath = path.join(__dirname, '../src');

  // compile the runner
  execute(`cd ${srcPath}/skw-vm-host && cargo check --release && cargo test --release`);
  execute(`cd ${srcPath}/skw-vm-engine && cargo check --release && cargo test --release`);
  execute(`cd ${srcPath}/skw-vm-engine-cli && cargo check --release && cargo test --release`);
}

build();
