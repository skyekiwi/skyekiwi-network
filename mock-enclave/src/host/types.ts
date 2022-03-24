// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

export type DBOps = {
  type: string,
  key: string,
  value: string,
}

export type CallRecord = [string, string]
export type SecretContractRegistrationEvent = [string]
export type IPFSResult = { cid: string, size: number };
