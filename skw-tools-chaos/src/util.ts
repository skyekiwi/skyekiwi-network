// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { SubmittableExtrinsic } from '@polkadot/api/promise/types';
import type { KeyringPair } from '@polkadot/keyring/types';

const sendTx = (
  extrinsic: SubmittableExtrinsic,
  sender: KeyringPair,
  logger: any,
): Promise<boolean> => {
  return new Promise((resolve, reject) => {
    extrinsic.signAndSend(sender, ({ events = [], status }) => {
      logger.info(
        `  ↪ 💸  Transaction status: ${status.type}`
      );

      if (
        status.isInvalid ||
        status.isDropped ||
        status.isUsurped ||
        status.isRetracted
      ) {
        console.error(status)
        reject(new Error('Invalid transaction'));
      } else {
        // Pass it
      }

      if (status.isInBlock) {
        events.forEach(({ event: { method, section } }) => {
          if (section === 'system' && method === 'ExtrinsicFailed') {
            // Error with no detail, just return error
            logger.error(`  ↪ ❌  Send transaction(${extrinsic.type}) failed.`);
            resolve(false);
          } else if (method === 'ExtrinsicSuccess') {
            logger.info(`  ↪ ✅  Send transaction(${extrinsic.type}) success.`);

            resolve(true);
          }
        });
      } else {
        // Pass it
      }
    }).catch(reject);
  });
};

export {sendTx}
