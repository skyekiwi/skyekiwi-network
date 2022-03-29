// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn railway:run', process.argv.slice(2).join(' '));

require('dotenv').config();

function runValidatorNode(seed, dbPath, p2pPort, wsPort, rpcPort, name, bootnodes,) {

  const node = path.join(__dirname, "../target/release/skyekiwi-node");

  console.log(seed)
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
    --port ${p2pPort} \
    --ws-port ${wsPort} \
    --rpc-port ${rpcPort} \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --validator \
    --rpc-methods Unsafe \
    --name ${name} \
    ${!!bootnodes ? `--bootnodes ${bootnodes}` : ''}`);
}


function runEndpoint(bootnodes) {
  const node = path.join(__dirname, "../target/release/skyekiwi-node");
  execSync(`${node} \
    --base-path ../tmp/db \
    --chain=skw_alpha \
    --port 30333 \
    --ws-port 443 \
    --rpc-port 9935 \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --rpc-methods Unsafe \
    --name Endpoint \
    --unsafe-ws-external \
    --rpc-cors=all \
    ${!!bootnodes ? `--bootnodes ${bootnodes}` : ''}`)
}

// /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWM2xBMj1ZmDV7JMKnjMtN7pAJwouogWiTYGFrSzv4iNSe
// EXPOSE 30333 9933 9944 9615

const main = () => {
  const mode = process.argv[2]
  const bootnode = process.env.BOOTNODES;

  switch (mode) {
    case 'endpoint':
      runEndpoint(bootnode);
      break;
    case 'validator':
      runValidatorAlone(0);
      break;
    default:
      runEndpoint(bootnode);
      break;
  }
}


function runValidatorAlone (num) {
  const dbPath = path.join(__dirname, `../tmp/db${num}`);
  const seed = process.env.SEED;
  const bootnode = process.env.BOOTNODES;
  runValidatorNode(seed, dbPath, 30333, 9944, 9933, `node-${num}`, bootnode);
}

main()