// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn main:build', process.argv.slice(2).join(' '));

function mainBuild() {
  execSync('cargo build --release');
}

mainBuild()

