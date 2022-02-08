// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
console.log('$ yarn railway:run', process.argv.slice(2).join(' '));

function railwayRun() {
  execSync(`./target/release/skyekiwi-node --tmp --dev`);
}

railwayRun()
