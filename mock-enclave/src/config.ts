// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { SContractConfiguration } from '@skyekiwi/s-contract/types';
import path from 'path'

export default {
  localStoragePath: path.join(__dirname, './mock/')
} as SContractConfiguration;
