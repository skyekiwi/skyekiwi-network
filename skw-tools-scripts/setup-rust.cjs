// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn setup-rust', process.argv.slice(2).join(' '));

function setupRust() {
  execSync('curl https://sh.rustup.rs -sSf | sh -s -- -y');
  execSync('source ~/.cargo/env');
  execSync('rustup default stable');
  execSync('rustup update nightly');
  execSync('rustup update stable');
  execSync('rustup target add wasm32-unknown-unknown --toolchain nightly');
  execSync('rustup target add wasm32-unknown-unknown');
}

setupRust()