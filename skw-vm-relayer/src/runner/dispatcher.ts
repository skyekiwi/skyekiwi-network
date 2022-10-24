import EventEmitter from "events";

import { initWASMInterface } from "@skyekiwi/crypto";

import { progressText, logger, DB } from "../util";
import { Dispatcher } from "../core";

const main = async() => {
  await initWASMInterface();
  const p = new EventEmitter();
  p.on("progress", (name: string, blockNumber?: number, callLength?: number) => {
    const text = progressText[name](blockNumber, callLength);
    logger.info(text);
  });

  const db = new DB();
  await db.init();

  const dispatcher = new Dispatcher(db, p);
  dispatcher.setShards([0]);

  void dispatcher.dispatchAll(); // no await 

  ['SIGINT', 'SIGTERM'].forEach(signal => {
    process.once(signal, () => {
      const shutdown = async() => {
        logger.warn("gracefully shutting down  ...")
        await dispatcher.shutdown();
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