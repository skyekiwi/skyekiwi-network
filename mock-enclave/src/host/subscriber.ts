import { ApiPromise } from '@polkadot/api';

import { getLogger } from '@skyekiwi/util';

/* eslint-disable sort-keys, camelcase, @typescript-eslint/ban-ts-comment */
export class Subscriber {
  public async subscribeNewBlock (
    api: ApiPromise,
    newBlockHook: (blockNumber: number) => Promise<void>,
  ): Promise<void> {
    const logger = getLogger('subscriber');
    logger.info("Subscribing to new blocks");
    await api.rpc.chain.subscribeNewHeads(async (latestHeader) => {
      logger.info(`new block received ${latestHeader.number.toNumber()}`);
      await newBlockHook(latestHeader.number.toNumber());
    });
  }
}
