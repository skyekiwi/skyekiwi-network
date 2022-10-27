import EventEmitter from "events";

import { initWASMInterface } from "@skyekiwi/crypto";
import { WsProvider, ApiPromise, Keyring } from "@polkadot/api";

import { DB, Chain, logger, progressText } from "../util";
import { Submitter } from "../core";

const main = async() => {
  await initWASMInterface();
  const p = new EventEmitter();
  p.on("progress", (name: string, blockNumber?: number, callLength?: number) => {
    const text = progressText[name](blockNumber, callLength);
    logger.info(text);
  });

  const seed = "//Alice" //process.env.SEED_PHRASE;

  if (!seed) {
    throw new Error('seed phrase not found');
  }
  const keyring = new Keyring({ type: 'sr25519' }).addFromUri(seed);

  const db = new DB();
  await db.init();
  await db.modify(db.createIndexerMetadata());

  const provider = new WsProvider('ws://localhost:8845');
  const api = await ApiPromise.create({ provider: provider });

  const chain = new Chain(api);

  const submitter = new Submitter(db,chain, keyring, p);
  submitter.setShards([0]);

  void submitter.parseAllSubmission(); // no await 

  ['SIGINT', 'SIGTERM'].forEach(signal => {
    process.once(signal, () => {
      const shutdown = async() => {
        logger.warn("gracefully shutting down  ...")
        await submitter.shutdown();
        await provider.disconnect();
      }

      shutdown().then(() => {
        process.exit(0);
      })
    });

    process.once(signal, () => {
      setTimeout(() => {
        logger.warn(`Could not close resources gracefully after 15s: forcing shutdown`);
        process.exit(1);
      }, 15000)
    })
  });
}
  
main();