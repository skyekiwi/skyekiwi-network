// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn contract:compile', process.argv.slice(2).join(' '));

function contractBuild() {
  execSync('cd skw-contract-sdk/skw-contract-sdk && cargo test --release');
}

contractBuild();
