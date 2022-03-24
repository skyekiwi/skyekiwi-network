// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { ApiPromise } from '@polkadot/api';

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Subscriber {
    public async subscribeNewBlock (
      api: ApiPromise,
      newBlockHook: (blockNumber: number) => Promise<void>,
    ): Promise<void> {
      await api.rpc.chain.subscribeNewHeads(async (latestHeader) => {
        await newBlockHook(latestHeader.number.toNumber());
      });
    }
}
