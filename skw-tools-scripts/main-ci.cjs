// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn blockchain:ci', process.argv.slice(2).join(' '));

function blockchainCi() {
  execSync('rustup target add wasm32-unknown-unknown');
  execSync('SKIP_WASM_BUILD=1 cargo check --release');
  execSync('SKIP_WASM_BUILD=1 cargo check --features=runtime-benchmarks --release');
  execSync('SKIP_WASM_BUILD=1 cargo test --release');
}

blockchainCi()
