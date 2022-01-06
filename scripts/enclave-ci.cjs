// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

const execSync = require('./execSync.cjs');
const path = require('path');
const fs = require('fs');

console.log('$ yarn enclave:ci', process.argv.slice(2).join(' '));

const sdk_bin = "https://download.01.org/intel-sgx/linux-2.6/ubuntu18.04-server/sgx_linux_x64_sdk_2.6.100.51363.bin"
const psw_deb = "https://download.01.org/intel-sgx/linux-2.6/ubuntu18.04-server/libsgx-enclave-common_2.6.100.51363-bionic1_amd64.deb"
const psw_dev_deb = "https://download.01.org/intel-sgx/linux-2.6/ubuntu18.04-server/libsgx-enclave-common-dev_2.6.100.51363-bionic1_amd64.deb"
const psw_dbgsym_deb = "https://download.01.org/intel-sgx/linux-2.6/ubuntu18.04-server/libsgx-enclave-common-dbgsym_2.6.100.51363-bionic1_amd64.ddeb"


function enclaveCI() {

  const pathName = path.join(__dirname, "../sgx-tmp");
  execSync(`sudo apt-get update`);
  execSync(`sudo apt-get install -y build-essential ocaml ocamlbuild automake autoconf libtool wget python libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev sudo kmod vim curl git-core libprotobuf-c0-dev libboost-thread-dev libboost-system-dev liblog4cpp5-dev libjsoncpp-dev alien uuid-dev libxml2-dev cmake pkg-config expect systemd-sysv gdb`);

  if (!fs.existsSync(pathName)) {
    execSync(`mkdir ${pathName}`);
  }

  execSync(`wget -O ${pathName}/psw.deb ${psw_deb} \
    && wget -O ${pathName}/psw_dev.deb ${psw_dev_deb} \
    && wget -O ${pathName}/psw_dbgsym.deb ${psw_dbgsym_deb}  \
    && wget -O ${pathName}/sdk.bin ${sdk_bin}\
    && cd ${pathName} \
    && sudo dpkg -i ${pathName}/psw.deb \
    && sudo dpkg -i ${pathName}/psw_dev.deb \
    && sudo dpkg -i ${pathName}/psw_dbgsym.deb \
    && chmod +x ${pathName}/sdk.bin \
    && echo -e 'no\n/opt' | ${pathName}/sdk.bin \
    && echo 'source /opt/sgxsdk/environment' >> ~/.bashrc \
    && rm -rf ${pathName}`
  );
  execSync(`cd enclave && make && cd bin && ./app`)
}

enclaveCI()
