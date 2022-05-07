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

function runValidatorNodeZero() {


  const dbPath = path.join(__dirname, `../tmp/db0`);
  const seed = 'dynamic lock electric bullet satisfy when figure vibrant two hurdle rent holiday'

  // insert the keys into localDB
  execSync(`${node} key insert \
    --base-path ${dbPath} \
    --chain=skw_alpha \
    --scheme Sr25519 \
    --suri "${seed}" \
    --key-type aura && \
  ${node} key insert \
    --base-path ${dbPath} \
    --chain=skw_alpha \
    --scheme Ed25519 \
    --suri "${seed}" \
    --key-type gran`);

  // start the node
  execSync(`${node} \
    --base-path ${dbPath} \
    --chain=skw_alpha \
    --port 30333 \
    --ws-port 9944 \
    --rpc-port 9935 \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --validator \
    --unsafe-rpc-external \
    --unsafe-ws-external 
    --prometheus-external `);
}

function runValidatorNodeOne() {
    const dbPath = path.join(__dirname, `../tmp/db1`);
    const seed = 'boil install know trigger sunset addict grit ozone shuffle airport steak photo'
    
    const bootnodes = '/ip4/0.0.0.0/tcp/30333/p2p/12D3KooWDRtewpSTpumfZ1MAxVUqvHr6tHW8759GsHa5tKWTNt3J';
    // insert the keys into localDB
    execSync(`${node} key insert \
      --base-path ${dbPath} \
      --chain=skw_alpha \
      --scheme Sr25519 \
      --suri "${seed}" \
      --key-type aura && \
    ${node} key insert \
      --base-path ${dbPath} \
      --chain=skw_alpha \
      --scheme Ed25519 \
      --suri "${seed}" \
      --key-type gran`);
  
    // start the node
    execSync(`${node} \
      --base-path ${dbPath} \
      --chain=skw_alpha \
      --port 30334 \
      --ws-port 9945 \
      --rpc-port 9936 \
      --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
      --validator \
      --unsafe-rpc-external \
      --unsafe-ws-external 
      --prometheus-external \
      --bootnodes ${bootnodes}`);
  }
  

const main = () => {
  const nodeNum = process.argv[2]
  if (nodeNum == 0) runValidatorNodeZero();
  else runValidatorNodeOne();
}
main()