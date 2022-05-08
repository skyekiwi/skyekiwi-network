// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn blockchain:benchmark', process.argv.slice(2).join(' '));

function generateBenchmark() {

  const node = path.join(__dirname, "../target/release/skyekiwi-node");
  const palletRootFolder = path.join(__dirname, "../crates/skw-blockchain-pallets");
  const weightTemplate = path.join(__dirname, "../misc/frame-weight-template.hbs");

  execSync(`${node} benchmark \
    --chain=skw_alpha \
    --steps=50 \
    --repeat=20 \
    --pallet=pallet_parentchain \
    --extrinsic='*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=${palletRootFolder}/pallet-parentchain/src/weights.rs \
    --template=${weightTemplate}`);

  execSync(`${node} benchmark \
    --chain=skw_alpha \
    --steps=50 \
    --repeat=20 \
    --pallet=pallet_registry \
    --extrinsic='*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=${palletRootFolder}/pallet-registry/src/weights.rs \
    --template=${weightTemplate}`);

  execSync(`${node} benchmark \
    --chain=skw_alpha \
    --steps=50 \
    --repeat=20 \
    --pallet=pallet_secrets \
    --extrinsic='*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=${palletRootFolder}/pallet-secrets/src/weights.rs \
    --template=${weightTemplate}`);

  execSync(`${node} benchmark \
    --chain=skw_alpha \
    --steps=50 \
    --repeat=20 \
    --pallet=pallet_s_contract \
    --extrinsic='*' \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output=${palletRootFolder}/pallet-s-contract/src/weights.rs \
    --template=${weightTemplate}`);
}

generateBenchmark()
