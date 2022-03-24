// Copyright 2021-2022 @skyekiwi/s-contract authors & contributors
// SPDX-License-Identifier: Apache-2.0

import BN from 'bn.js';
import { baseDecode, baseEncode, deserialize, serialize } from 'borsh';

/* eslint-disable sort-keys, camelcase */
class Call {
  public origin: string
  public origin_public_key: Uint8Array

  public encrypted_egress: boolean

  public transaction_action: 'create_account' | 'call' | 'transfer' | 'view_method_call' | 'deploy'
  public receiver: string
  public amount: BN | null
  public wasm_blob_path: string | null
  public method: string | null
  public args: string | null
  public to: string | null

  constructor (config: {
    origin: string,
    origin_public_key: Uint8Array,

    encrypted_egress: boolean,

    transaction_action: 'create_account' | 'call' | 'transfer' | 'view_method_call' | 'deploy',
    receiver: string,
    amount: BN | null,
    wasm_blob_path: string | null,
    method: string | null,
    args: string | null,
    to: string | null,
  }) {
    this.origin = config.origin;
    this.origin_public_key = config.origin_public_key;
    this.encrypted_egress = config.encrypted_egress;

    this.transaction_action = config.transaction_action;
    this.receiver = config.receiver;
    this.amount = config.amount;
    this.wasm_blob_path = config.wasm_blob_path;
    this.method = config.method;
    this.args = config.args;
    this.to = config.to;
  }
}

class Calls {
  public ops: Call[]
  constructor (config: {
    ops: Call[],
  }) {
    this.ops = config.ops;
  }
}
class Outcome {
  public view_result_log: string[]
  public view_result: Uint8Array
  public outcome_logs: string[]
  public outcome_receipt_ids: Uint8Array[]
  public outcome_gas_burnt: BN
  public outcome_token_burnt: BN
  public outcome_executor_id: string
  public outcome_status: Uint8Array | null

  constructor (config: {
    view_result_log: string[],
    view_result: Uint8Array,
    outcome_logs: string[],
    outcome_receipt_ids: Uint8Array[],
    outcome_gas_burnt: BN,
    outcome_token_burnt: BN,
    outcome_executor_id: string,
    outcome_status: Uint8Array | null,
  }) {
    this.view_result_log = config.view_result_log;
    this.view_result = config.view_result;
    this.outcome_logs = config.outcome_logs;
    this.outcome_receipt_ids = config.outcome_receipt_ids;
    this.outcome_gas_burnt = config.outcome_gas_burnt;
    this.outcome_token_burnt = config.outcome_token_burnt;
    this.outcome_executor_id = config.outcome_executor_id;
    this.outcome_status = config.outcome_status;
  }
}

class Outcomes {
  public ops: Outcome[]
  public state_root: Uint8Array

  constructor (config: {
    ops: Outcome[],
    state_root: Uint8Array
  }) {
    this.ops = config.ops;
    this.state_root = config.state_root;
  }
}
class OutcomesOutput {
  public ops: Outcome[]
  public callIndex: number

  constructor (config: {
    ops: Outcome[],
    callIndex: number
  }) {
    this.ops = config.ops;
    this.callIndex = config.callIndex;
  }
}

class LocalMetadata {
  public shard_id: number[]
  public high_local_block: number
  public latest_state_root: Uint8Array

  constructor (config: {
    shard_id: number[],
    high_local_block: number,
    latest_state_root: Uint8Array,
  }) {
    this.shard_id = config.shard_id;
    this.high_local_block = config.high_local_block;
    this.latest_state_root = config.latest_state_root;
  }
}

class Block {
  public shard_id: number
  public block_number: number
  public calls: number[]
  public contracts: string[]

  constructor (config: {
    shard_id: number,
    block_number: number,
    calls: number[],
    contracts: string[],
  }) {
    this.shard_id = config.shard_id;
    this.block_number = config.block_number;
    this.calls = config.calls;
    this.contracts = config.contracts;
  }
}

class Shard {
  public high_remote_synced_block_index: number
  public high_remote_confirmed_block_index: number

  constructor (config: {
    high_remote_synced_block_index: number,
    high_remote_confirmed_block_index: number,
  }) {
    this.high_remote_synced_block_index = config.high_remote_synced_block_index;
    this.high_remote_confirmed_block_index = config.high_remote_confirmed_block_index;
  }
}

class ShardMetadata {
  public shard_key: Uint8Array
  public shard_members: string[]
  public beacon_index: number
  public threshold: number

  constructor (config: {
    shard_key: Uint8Array,
    shard_members: string[],
    beacon_index: number,
    threshold: number,
  }) {
    this.shard_key = config.shard_key;
    this.shard_members = config.shard_members;
    this.beacon_index = config.beacon_index;
    this.threshold = config.threshold;
  }
}

class Contract {
  public home_shard: number
  public wasm_blob: string
  public deployment_call: Calls
  public deployment_call_index: number

  constructor (config: {
    home_shard: number,
    wasm_blob: string,
    deployment_call: Calls,
    deployment_call_index: number,
  }) {
    this.home_shard = config.home_shard;
    this.wasm_blob = config.wasm_blob;
    this.deployment_call = config.deployment_call;
    this.deployment_call_index = config.deployment_call_index;
  }
}

class ExecutionSummary {
  public high_local_execution_block: number

  constructor (config: {
    high_local_execution_block: number,
  }) {
    this.high_local_execution_block = config.high_local_execution_block;
  }
}

const blockSchema = new Map([
  [Block, {
    kind: 'struct',
    fields: [
      ['shard_id', 'u32'],
      ['block_number', 'u32'],
      ['calls', { kind: 'option', type: ['u32'] }],
      ['contracts', { kind: 'option', type: ['string'] }]
    ]
  }]
]);

const callSchema = new Map([
  [Call, {
    kind: 'struct',
    fields: [
      ['origin', { kind: 'option', type: 'string' }],
      ['origin_public_key', { kind: 'option', type: ['u8', 32] }],

      ['encrypted_egress', 'u8'],

      ['transaction_action', 'string'],
      ['receiver', 'string'],
      ['amount', { kind: 'option', type: 'u128' }],
      ['wasm_blob_path', { kind: 'option', type: 'string' }],
      ['method', { kind: 'option', type: 'string' }],
      ['args', { kind: 'option', type: 'string' }],
      ['to', { kind: 'option', type: 'string' }]
    ]
  }]
]);
const callsSchema = new Map();

callsSchema.set(Calls, {
  kind: 'struct',
  fields: [
    ['ops', [Call]],
  ]
});
callsSchema.set(Call, callSchema.get(Call));

const outcomeSchema = new Map([[Outcome, {
  kind: 'struct',
  fields: [
    ['view_result_log', ['string']],
    ['view_result', ['u8']],
    ['outcome_logs', ['string']],
    ['outcome_receipt_ids', [['u8', 32]]],
    ['outcome_gas_burnt', 'u64'],
    ['outcome_token_burnt', 'u128'],
    ['outcome_executor_id', 'String'],
    ['outcome_status', { kind: 'option', type: ['u8'] }]
  ]
}]]);
const outcomesSchema = new Map();

outcomesSchema.set(Outcomes, {
  kind: 'struct',
  fields: [
    ['ops', [Outcome]],
    ['state_root', ['u8', 32]]
  ]
});
outcomesSchema.set(Outcome, outcomeSchema.get(Outcome));

const outcomesOutputSchema = new Map();

outcomesOutputSchema.set(OutcomesOutput, {
  kind: 'struct',
  fields: [
    ['ops', [Outcome]],
    ['call_index', 'u32'],
  ]
});
outcomesOutputSchema.set(Outcomes, outcomesSchema.get(Outcomes));
outcomesOutputSchema.set(Outcome, outcomeSchema.get(Outcome));

const contractSchema = new Map();

contractSchema.set(Contract, {
  kind: 'struct',
  fields: [
    ['home_shard', 'u32'],
    ['wasm_blob', 'string'],
    ['deployment_call', Calls],
    ['home_shard', 'u32'],
  ]
});
contractSchema.set(Calls, callsSchema.get(Calls));
contractSchema.set(Call, callSchema.get(Call));

const shardMetadataSchema = new Map([
  [ShardMetadata, {
    kind: 'struct',
    fields: [
      ['shard_key', ['u8', 32]],
      ['shard_members', ['string']],
      ['beacon_index', 'u32'],
      ['threshold', 'u32']
    ]
  }]
]);
const shardSchema = new Map([
  [Shard, {
    kind: 'struct',
    fields: [
      ['high_remote_synced_block_index', 'u32'],
      ['high_remote_confirmed_block_index', 'u32']
    ]
  }]
]);
const localMetadataSchema = new Map([
  [LocalMetadata, {
    kind: 'struct',
    fields: [
      ['shard_id', ['u32']],
      ['high_local_block', 'u32'],
      ['latest_state_root', ['u8', 32]],
    ]
  }]
]);
const executionSummarySchema = new Map([
  [ExecutionSummary, {
    kind: 'struct',
    fields: [
      ['high_local_execution_block', 'u32']
    ]
  }]
]);

// ser
const buildCall = (
  call: Call
): string => {
  const buf = serialize(callSchema, call);

  return baseEncode(buf);
};

const buildBlock = (
  block: Block
): string => {
  const buf = serialize(blockSchema, block);

  return baseEncode(buf);
};

const buildContract = (
  contract: Contract
): string => {
  const buf = serialize(contractSchema, contract);

  return baseEncode(buf);
};

const buildCalls = (
  calls: Calls
): string => {
  if (calls.ops.length === 0) {
    return '';
  }

  const buf = serialize(callsSchema, calls);

  return baseEncode(buf);
};

const buildOutcome = (
  outcome: Outcome
): string => {
  const buf = serialize(outcomeSchema, outcome);

  return baseEncode(buf);
};

const buildOutcomes = (
  outcomes: Outcomes
): string => {
  const buf = serialize(outcomesSchema, outcomes);

  return baseEncode(buf);
};

const buildShard = (
  shard: Shard
): string => {
  const buf = serialize(shardSchema, shard);

  return baseEncode(buf);
};

const buildShardMetadata = (
  shardMetadata: ShardMetadata
): string => {
  const buf = serialize(shardMetadataSchema, shardMetadata);

  return baseEncode(buf);
};

const buildLocalMetadata = (a: LocalMetadata): string => {
  const buf = serialize(localMetadataSchema, a);

  return baseEncode(buf);
};

const buildExecutionSummary = (a: ExecutionSummary): string => {
  const buf = serialize(executionSummarySchema, a);

  return baseEncode(buf);
};

const buildOutcomesOutput = (a: OutcomesOutput): string => {
  const buf = serialize(outcomesOutputSchema, a);
  return baseEncode(buf);
};

// de
const parseCall = (
  buf: string
): Call => {
  const c = deserialize(callSchema, Call, baseDecode(buf));

  c.encrypted_egress = !!c.encrypted_egress;

  return c;
};

const parseCalls = (
  buf: string
): Calls => {
  if (buf.length === 0) {
    return { ops: [] };
  }

  const cs = deserialize(callsSchema, Calls, baseDecode(buf));

  for (let i = 0; i < cs.ops.length; i++) {
    cs.ops[i].encrypted_egress = !!cs.ops[i].encrypted_egress;
  }

  return cs;
};

const parseOutcome = (
  buf: string
): Outcome => {
  return deserialize(outcomeSchema, Outcome, baseDecode(buf));
};

const parseOutcomes = (
  buf: string
): Outcomes => {
  return deserialize(outcomesSchema, Outcomes, baseDecode(buf));
};

const parseBlock = (
  buf: string
): Block => {
  return deserialize(blockSchema, Block, baseDecode(buf));
};

const parseContract = (
  buf: string
): Contract => {
  return deserialize(contractSchema, Contract, baseDecode(buf));
};

const parseShard = (
  buf: string
): Shard => {
  return deserialize(shardSchema, Shard, baseDecode(buf));
};

const parseShardMetadata = (
  buf: string
): ShardMetadata => {
  return deserialize(shardMetadataSchema, ShardMetadata, baseDecode(buf));
};

const parseLocalMetadata = (buf: string): LocalMetadata => {
  return deserialize(localMetadataSchema, LocalMetadata, baseDecode(buf));
};

const parseExecutionSummary = (
  buf: string
): ExecutionSummary => {
  return deserialize(executionSummarySchema, ExecutionSummary, baseDecode(buf));
};

const parseOutcomesOutput = (
  buf: string
): OutcomesOutput => {
  return deserialize(outcomesOutputSchema, OutcomesOutput, baseDecode(buf));
};

export {
  Call, callSchema, buildCall, parseCall,
  Calls, callsSchema, buildCalls, parseCalls,
  Outcome, outcomeSchema, buildOutcome, parseOutcome,
  Outcomes, outcomesSchema, buildOutcomes, parseOutcomes,
  OutcomesOutput, outcomesOutputSchema, buildOutcomesOutput, parseOutcomesOutput,
  Block, blockSchema, buildBlock, parseBlock,
  Contract, contractSchema, buildContract, parseContract,
  Shard, shardSchema, buildShard, parseShard,
  ShardMetadata, shardMetadataSchema, buildShardMetadata, parseShardMetadata,
  LocalMetadata, localMetadataSchema, buildLocalMetadata, parseLocalMetadata,
  ExecutionSummary, executionSummarySchema, buildExecutionSummary, parseExecutionSummary
};
