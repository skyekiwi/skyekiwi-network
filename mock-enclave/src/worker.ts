// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { RequestInitializeContract } from '@skyekiwi/s-contract/types'

import { expose } from 'threads/worker';
import { getLogger, indexToString, stringToIndex, u8aToHex } from '@skyekiwi/util'
import { File } from '@skyekiwi/file'
import { SContractPersistent, SContractReader } from '@skyekiwi/s-contract';
import { DefaultSealer } from '@skyekiwi/crypto'
import { decodeAddress } from '@polkadot/util-crypto'

import {runVM} from './vm'

import fs from 'fs'
import configuration from './config'

let instance: SContractReader;
let currentContractId: string;

let currentHighRemoteCallIndex: string
let currentHighLocalCallIndex: string

require('dotenv').config();

const enclaveMock = {
  async initialzeContract(request: RequestInitializeContract) {
    const {contractId, wasmBlob, highRemoteCallIndex} = request;

    const logger = getLogger("enclaveMock.initializeContract");

    logger.info(`initializing contract id ${contractId}`);

    const outputPath = `${configuration.localStoragePath}${contractId}.contract`;

    currentContractId = contractId;
    try {
      const file = new File({
        fileName: 'contract',
        readStream: fs.createReadStream(outputPath)
      });
      instance = new SContractReader(file, new DefaultSealer())
      await instance.init();
      instance.unlockSealer(process.env.SEED_PHRASE);

      if (instance.readContract() === undefined) {
        throw new Error(`local contract ${contractId} not found, fetching from remote`)
      }
    } catch (err) {
      logger.warn(err);
      logger.info(`Local Contract not found - downstreaming now`)

      instance = await SContractPersistent.initialize(
        configuration, contractId, wasmBlob
      )

      if (stringToIndex(currentHighLocalCallIndex) < stringToIndex(highRemoteCallIndex)) {
        logger.info(`local callIndex ${currentHighLocalCallIndex} is lower than the remote callIndex ${highRemoteCallIndex}, initialize rolldown request`);
      }
    }

    currentHighLocalCallIndex = instance.getHighLocalCallIndex();
    currentHighRemoteCallIndex = highRemoteCallIndex;

    logger.info(`contract public key ${u8aToHex(instance.getContractPublicKey())}`)
    logger.info(`contract ${contractId} initialization success`)
  },
  async dispatchCall(_call: string): Promise<void> {
    const logger = getLogger('enclaveMock.dispatchCall');

    const currentHighLocalCallIndexNumber = stringToIndex(currentHighLocalCallIndex)
    try {
      const call = instance.decodeCall(_call);
      logger.info(`dispatched call ${call.callIndex}, local call ${currentHighLocalCallIndex}`);

      const callIndexNumber = stringToIndex(call.callIndex);
      if (callIndexNumber === currentHighLocalCallIndexNumber + 1) {

        const config = {
          methodName: call.methodName,
          stateInput: JSON.stringify(instance.readState()),
          input: JSON.stringify(call.parameters),
          wasmFile: `${configuration.localStoragePath}${call.contractId}.wasm`,
          origin: u8aToHex(decodeAddress(call.origin)),
          profiling: false,
          contractId: currentContractId
        }

        if (instance.readState() === "{}")
          delete config['stateInput']

        const nextState = runVM(config);
        logger.info(JSON.stringify(nextState));

        instance.writeState(JSON.stringify(nextState))

        currentHighLocalCallIndex = indexToString(currentHighLocalCallIndexNumber + 1);
      } else {
        if (callIndexNumber > currentHighLocalCallIndexNumber) {
          throw new Error(`needs rolldown, local execution queue too low`);
        } else {
          logger.error(`unexpected index - local ${currentHighLocalCallIndexNumber} & remote ${callIndexNumber}`);
          // pass
        }
      }

      logger.info(`contract ${currentContractId} executing call ${call.callIndex} done`)
    } catch(err) {
      logger.warn(err)
      // pass
    }    
  },
}

expose(enclaveMock)
