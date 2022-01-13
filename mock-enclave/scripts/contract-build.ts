// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import fs from 'fs';
import { fromByteArray, toByteArray } from 'base64-js';
import { u8aToString } from '@skyekiwi/util';

const { execute } = require('./execSync');

console.log('$ yarn build', process.argv.slice(2).join(' '));

function build() {
  // compile the runner
  execute('cd contract && cargo build --target wasm32-unknown-unknown --release');
  execute('cp contract/target/wasm32-unknown-unknown/release/greeting.wasm wasm/');
}

build();
