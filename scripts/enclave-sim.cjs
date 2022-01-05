// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn enclave:sim', process.argv.slice(2).join(' '));

function runInDocker() {

  const pathName = path.join(__dirname, "..");
  execSync(`sudo docker run -v ${pathName}:/root/sgx -it baiduxlab/sgx-rust bash -c "cd /root/sgx/enclave && export SGX_MODE=SW && bash"`);
}

runInDocker()

