import { execSync } from 'child_process';
import path from 'path';

const main = async() => {

  const pm2Path = path.join(__dirname, "../node_modules/.bin/pm2");
  const tsNodePath = path.join(__dirname, "../node_modules/.bin/ts-node");
  const runnerPath = path.join(__dirname, "./runner");
  const vmInterfacePath = path.join(__dirname, "../../crates/skw-vm-interface");

  // Launch VM Interface
  execSync( `${pm2Path} start "cd ${vmInterfacePath} && cargo run"` );

  // Launch DB
  execSync( `${pm2Path} start "surreal start --log debug --user root --pass root --bind 0.0.0.0:8081 file://${path.join(__dirname, "../local")}"` );

  // Launch Indexer
  execSync( `${pm2Path} start "${tsNodePath} ${runnerPath}/indexer"` );

  // Launch Dispatcher
  execSync( `${pm2Path} start "${tsNodePath} ${runnerPath}/dispatcher"` );

  // Launch Submitter
  execSync( `${pm2Path} start "${tsNodePath} ${runnerPath}/submitter"` );
}

main();

