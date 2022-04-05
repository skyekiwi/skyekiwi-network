// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { SubmittableExtrinsic } from "@polkadot/api/promise/types"

export type DBOps = {
  type: string,
  key: string,
  value: string,
}

export type CallRecord = [string, string]
export type SecretContractRegistrationEvent = [string]
export type QueuedTransaction = {
  transaction: SubmittableExtrinsic,
  blockNumber: number,
}