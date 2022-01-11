// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import { Chaos } from './chaos'

const main = async () => {
  const h = new Chaos();

  // let account //X to push Y random calls
  await h.letsParty(Number(process.argv[2]), Number(process.argv[3]))
}

main()
