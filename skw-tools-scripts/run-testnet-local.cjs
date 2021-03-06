// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn railway:run', process.argv.slice(2).join(' '));

try {
  require('dotenv').config();
} catch (e) {
  // pass, deplying on railway
}

const node = path.join(__dirname, "../target/release/skyekiwi-node");

const seedsRaw = process.env.SEEDS;
if (!seedsRaw) {
  throw new Error("Seed phrase not found")
}

const seeds = JSON.parse(seedsRaw);

function runValidatorNodeZero() {
  const dbPath = path.join(__dirname, `../tmp/db0`);
  const seed = seeds[0];

  // insert the keys into localDB
  execSync(`${node} key insert \
    --base-path ${dbPath} \
    --scheme Sr25519 \
    --suri "${seed}" \
    --key-type aura && \
  ${node} key insert \
    --base-path ${dbPath} \
    --scheme Ed25519 \
    --suri "${seed}" \
    --key-type gran`);

  // start the node
  execSync(`${node} \
    --base-path ${dbPath} \
    --port 30333 \
    --ws-port 9944 \
    --rpc-port 9935 \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --validator \
    --rpc-cors all \
    --unsafe-rpc-external \
    --unsafe-ws-external \
    --prometheus-external `);
}

function runValidatorNodeOne() {
    const dbPath = path.join(__dirname, `../tmp/db1`);
    const seed = seeds[1]
    
    const bootnodes = process.env.BOOTNODES;
    if (!bootnodes) {
      throw new Error("Bootnodes not found")
    }
        // insert the keys into localDB
    execSync(`${node} key insert \
      --base-path ${dbPath} \
      --scheme Sr25519 \
      --suri "${seed}" \
      --key-type aura && \
    ${node} key insert \
      --base-path ${dbPath} \
      --scheme Ed25519 \
      --suri "${seed}" \
      --key-type gran`);
  
    // start the node
    execSync(`${node} \
      --base-path ${dbPath} \
      --port 30333 \
      --ws-port 9944 \
      --rpc-port 9935 \
      --rpc-cors all \
      --validator \
      --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
      --unsafe-rpc-external \
      --unsafe-ws-external \
      --bootnodes ${bootnodes}`);
}

function runFullnode() {
  const dbPath = path.join(__dirname, `../tmp/fullnode`);

  // start the node
  execSync(`${node} \
    --base-path ${dbPath} \
    --port 30333 \
    --ws-port 9944 \
    --rpc-port 9935 \
    --rpc-cors all \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --unsafe-rpc-external \
    --unsafe-ws-external`);
}
  

const main = () => {
  const nodeNum = process.argv[2]
  if (nodeNum == 0) runValidatorNodeZero();
  else if (nodeNum == 1) runValidatorNodeOne();
  else runFullnode();
}
main()