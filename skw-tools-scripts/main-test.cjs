// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn main:test', process.argv.slice(2).join(' '));

function blockchainCi() {
  const module = process.argv[2] ? `-p ${process.argv[2]}` : "";

  execSync(`cargo test ${module} --release`);
}

blockchainCi()
