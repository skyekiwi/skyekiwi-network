import EventEmitter from "events";

import { initWASMInterface } from "@skyekiwi/crypto";
import { WsProvider, ApiPromise, Keyring } from "@polkadot/api";

import { progressText, logger, DB, Chain } from "../util";
import { Indexer } from "../core";

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

  const indexer = new Indexer(db, p);
  indexer.setShards([0]);

  void indexer.fetch(chain); // no await 
  void indexer.fetchShardInfo(chain, keyring.address); // no await

  ['SIGINT', 'SIGTERM'].forEach(signal => {
    process.once(signal, () => {
      const shutdown = async() => {
        logger.warn("gracefully shutting down  ...")
        await indexer.shutdown();
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