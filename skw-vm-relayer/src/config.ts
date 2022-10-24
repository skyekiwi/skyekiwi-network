// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path';

export default {
  vmBinaryPath: path.join(__dirname, '../../target/release/skw-vm-interface'),
  statePatcherBinaryPath: path.join(__dirname, '../target/release/skw-vm-patch'),
  localStoragePath: path.join(__dirname, './mock/'),
  stateDumpPrefix: path.join(__dirname, "../../vm-state-dump/interface"), 
  genesisStateFile: path.join(__dirname, "../../vm-state-dump/empty__state_dump__ColState"),
  currentStateFile: path.join(__dirname, "../../vm-state-dump/interface__state_dump__ColState"),
  localWASMStorage: path.join(__dirname, "../wasm"),

  surrealDBPort: 8081,
  surrealDBEndpoint: "http://127.0.0.1:8081/rpc",
  enclaveRunnerEndpoint: "http://127.0.0.1:8000",
};


