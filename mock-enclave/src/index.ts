import { SContractHost } from './host';
import {preRun} from './vm';

const main = async() => {
  // compile
  preRun()

  const host = new SContractHost();
  await host.mockMainLoop(100);
}

main().finally(() => {
  console.log("subscribing to events ... ")
})
