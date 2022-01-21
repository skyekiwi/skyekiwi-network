// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
console.log('$ yarn enclave:ci', process.argv.slice(2).join(' '));

function enclaveCI() {

  const pathName = path.join(__dirname, "..");
  execSync(`sudo docker run -v ${pathName}:/root/sgx \
    baiduxlab/sgx-rust:1804-1.1.4 \
    bash -c " \
      source /opt/sgxsdk/environment \
      && source /root/.cargo/env \
      && cd /root/sgx/enclave \
      && export SGX_MODE=SW \
      && make \
      && cd bin \
      && ./skw_sgx_host \
      && exit"`
  );
}

enclaveCI()
