// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn blockchain:build', process.argv.slice(2).join(' '));

function buildBlockchain() {
  execSync('cargo build --release');
}

buildBlockchain()

