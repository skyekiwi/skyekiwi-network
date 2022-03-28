// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import path from 'path';

export default {
  localStoragePath: path.join(__dirname, './mock/'),
  stateDumpPrefix: path.join(__dirname, "../../vm-state-dump/interface"), 
  genesisStateFile: path.join(__dirname, "../../vm-state-dump/empty__state_dump__ColState"),
  currentStateFile: path.join(__dirname, "../../vm-state-dump/interface__state_dump__ColState"),

};


