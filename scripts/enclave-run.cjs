// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn enclave:run', process.argv.slice(2).join(' '));

function runEnclave() {
  execSync('cd src/bin && RUST_LOG=trace ./app')
}

runEnclave()
