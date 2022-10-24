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

export type BlockRawCalls = [[number, Uint8Array, string]]; // callIndex, encodedCall, sender

// DB Types
export type ShardInfo = {
  shard_member: string[],
  beacon_index: number,
  threshold: number,
  id: string
};

export type CallsInDB = {
  encoded: string,
  call_index: number,
  origin: string,
}

export type OutcomesInDB = {
  encoded: string,
  call_index: number;
};

export type Block = {
  block_number: number, 
  shard_id: number,
  calls: number[],
  state_root: string,
  outcomes: number[]
};

export type ExpandedBlockForCall = {
  block_number: number,
  shard_id: number,
  calls: CallsInDB[],
  state_root: string,
}

export type ExpandedBlockForOutcomes = {
  block_number: number,
  shard_id: number,
  state_root: string,
  outcomes: OutcomesInDB[],
}

export type BuffedTx = {
  encoded_tx: string,
  status: string,
}