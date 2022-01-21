// Copyright 2021 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { RequestDispatch, RequestInitializeContract, RequestRolldown } from '@skyekiwi/s-contract/types';

import { getLogger, indexToString } from '@skyekiwi/util';
import { spawn, Worker } from 'threads'
import { MockBlockchainEnv } from './blockchain';
import PQueue from 'p-queue';

const contractId = '0x0001b4';
const contractId2 = indexToString(467);
const wasmBlobCID = 'Qmc8aHYGquRWeheiQ51xP3Z6EsyQWD89XSVTrQRfQWdFcA';

require('dotenv').config();

// Host filter incoming calls and route accordingly
export class SContractHost {
  #instances: { [key: string]: any }

  // 0 -> not init; 1 -> waiting to be inited; 2 -> good to go
  #isReady: { [key: string]: number }
  #initializationQueue: { [key: string]: any }
  #queue: PQueue

  constructor() {
    this.#instances = {};
    this.#initializationQueue = {}
    this.#isReady = {}
    this.#queue = new PQueue({concurrency: 1})
  }

  public async mockMainLoop(blockNum: number) {
    const logger = getLogger('SContractHost.mockMainLoop');

    const mock = new MockBlockchainEnv(blockNum);
    try {
      mock.spawnNewContractReq(this.subscribeInitializeContracts.bind(this), contractId, wasmBlobCID);
      mock.spawnNewContractReq(this.subscribeInitializeContracts.bind(this), contractId2, wasmBlobCID);

      for (const contractId in this.#instances) {
        this.#instances[contractId] = await this.#instances[contractId]

        let retries = 10;
        while(retries --) {
          try {
            await this.#instances[contractId].initialzeContract(this.#initializationQueue[contractId])
            delete this.#initializationQueue[contractId];
            break;
          } catch (err) {
            // pass
            logger.warn(err)
          }
        }
        
        this.#isReady[contractId] = 2
      }

      mock.spawnBlocks(this.subscribeDispatch.bind(this), contractId);
      mock.spawnBlocks(this.subscribeDispatch.bind(this), contractId2);
    } catch (err) {
      logger.error(err);
    }

  }

  public subscribeInitializeContracts(request: RequestInitializeContract) {
    const logger = getLogger('SContractHost.subscribeInitializeContracts');
    const { contractId } = request;

    // 1. filter
    if (this.#instances.hasOwnProperty(contractId)) {
      logger.info(`Request for initialize new contract ${contractId} receiverd, but the contract is already initialized. Passing`);
      // pass
    } else {
      const pool = spawn(new Worker('./worker'));
  
      this.#initializationQueue[contractId];
      logger.info(`pushing the initialzation request to the tasks queue ${JSON.stringify(request)} `)
      
      this.#initializationQueue[contractId] = request;
      this.#instances[contractId] = pool;
      this.#isReady[contractId] = 1;
    }
  }

  public subscribeDispatch(request: RequestDispatch): void {
    const { calls } = request;
    const logger = getLogger('SContractHost.subscribeDispatch');


    for (const call of calls) {
      const currentContractId = call.contractId;

      if (!this.#instances.hasOwnProperty(currentContractId)) {
        logger.error(`trying to execute contract ${currentContractId}, but no local instance is available`)
      }
      // 
      this.#queue.add(async() => await this.#instances[currentContractId].dispatchCall(call.call));
    }
  }

  public async subscribeRolldown(request: RequestRolldown): Promise<void> {
    const { contractId, highLocalCallIndex, highRemoteCallIndex } = request;
    const logger = getLogger('SContractHost.subscribeRolldown');

    logger.info(`should rolldown ${contractId}, from ${highLocalCallIndex} to ${highRemoteCallIndex}`);
  }
}
