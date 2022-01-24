use std::collections::{HashMap, HashSet};

use borsh::BorshSerialize;

use skw_vm_genesis_configs::Genesis;
use skw_vm_primitives::crypto::PublicKey;
use skw_vm_primitives::fees::StorageUsageConfig;
use skw_vm_primitives::{
    contract_runtime::{ContractCode, AccountId, Balance, MerkleHash, StateChangeCause, StateRoot},
    receipt::{DelayedReceiptIndices, Receipt, ReceiptEnum, ReceivedData},
    state_record::{state_record_to_account_id, StateRecord},
    trie_key::TrieKey,
};
use skw_vm_store::{
    get_account, get_received_data, set, set_account, set_code,
    set_postponed_receipt, set_received_data, ShardTries, TrieUpdate,
};

use crate::config::RuntimeConfig;
use crate::Runtime;

pub struct StorageComputer<'a> {
    result: HashMap<AccountId, u64>,
    config: &'a StorageUsageConfig,
}

impl<'a> StorageComputer<'a> {
    pub fn new(config: &'a RuntimeConfig) -> Self {
        Self { result: HashMap::new(), config: &config.transaction_costs.storage_usage_config }
    }

    pub fn process_record(&mut self, record: &StateRecord) {
        let account_and_storage = match record {
            StateRecord::Account { account_id, .. } => {
                Some((account_id.clone(), self.config.num_bytes_account))
            }
            StateRecord::Data { account_id, data_key, value } => {
                let storage_usage =
                    self.config.num_extra_bytes_record + data_key.len() as u64 + value.len() as u64;
                Some((account_id.clone(), storage_usage))
            }
            StateRecord::Contract { account_id, code } => {
                Some((account_id.clone(), code.len() as u64))
            }
            StateRecord::PostponedReceipt(_) => None,
            StateRecord::ReceivedData { .. } => None,
            StateRecord::DelayedReceipt(_) => None,
        };
        if let Some((account_id, storage_usage)) = account_and_storage {
            *self.result.entry(account_id).or_default() += storage_usage;
        }
    }

    pub fn process_records(&mut self, records: &[StateRecord]) {
        for record in records {
            self.process_record(record);
        }
    }

    pub fn finalize(self) -> HashMap<AccountId, u64> {
        self.result
    }
}

pub struct GenesisStateApplier {}

impl GenesisStateApplier {
    fn commit(
        mut state_update: TrieUpdate,
        current_state_root: &mut StateRoot,
        tries: &mut ShardTries,
    ) {
        state_update.commit(StateChangeCause::InitialState);
        let trie_changes = state_update.finalize_genesis().expect("Genesis state update failed");

        let (store_update, new_state_root) =
            tries.apply_all(&trie_changes).expect("Failed to apply genesis chunk");
        store_update.commit().expect("Store update failed on genesis initialization");
        *current_state_root = new_state_root;
    }

    fn apply_batch(
        current_state_root: &mut StateRoot,
        delayed_receipts_indices: &mut DelayedReceiptIndices,
        tries: &mut ShardTries,
        config: &RuntimeConfig,
        genesis: &Genesis,
        batch_account_ids: HashSet<&AccountId>,
    ) {
        let mut state_update = tries.new_trie_update(*current_state_root);
        let mut postponed_receipts: Vec<Receipt> = vec![];

        let mut storage_computer = StorageComputer::new(config);

        genesis.for_each_record(|record: &StateRecord| {
            if !batch_account_ids.contains(state_record_to_account_id(record)) {
                return;
            }

            storage_computer.process_record(record);

            match record.clone() {
                StateRecord::Account { account_id, account } => {
                    set_account(&mut state_update, account_id, &account);
                }
                StateRecord::Data { account_id, data_key, value } => {
                    state_update.set(TrieKey::ContractData { key: data_key, account_id }, value);
                }
                StateRecord::Contract { account_id, code } => {
                    let acc = get_account(&state_update, &account_id).expect("Failed to read state").expect("Code state record should be preceded by the corresponding account record");
                    // Recompute contract code hash.
                    let code = ContractCode::new(&code);
                    set_code(&mut state_update, account_id, &code);
                    assert_eq!(code.hash, acc.code_hash());
                }
                StateRecord::PostponedReceipt(receipt) => {
                    // Delaying processing postponed receipts, until we process all data first
                    postponed_receipts.push(*receipt);
                }
                StateRecord::ReceivedData { account_id, data_id, data } => {
                    set_received_data(
                        &mut state_update,
                        account_id,
                        data_id,
                        &ReceivedData { data },
                    );
                }
                StateRecord::DelayedReceipt(receipt) => {
                    Runtime::delay_receipt(
                        &mut state_update,
                        delayed_receipts_indices,
                        &*receipt,
                    )
                        .unwrap();
                }
            }
        });

        for (account_id, storage_usage) in storage_computer.finalize() {
            let mut account = get_account(&state_update, &account_id)
                .expect("Genesis storage error")
                .expect("Account must exist");
            account.set_storage_usage(storage_usage);
            set_account(&mut state_update, account_id, &account);
        }

        // Processing postponed receipts after we stored all received data
        for receipt in postponed_receipts {
            let account_id = &receipt.receiver_id;
            let action_receipt = match &receipt.receipt {
                ReceiptEnum::Action(a) => a,
                _ => panic!("Expected action receipt"),
            };
            // Logic similar to `apply_receipt`
            let mut pending_data_count: u32 = 0;
            for data_id in &action_receipt.input_data_ids {
                if get_received_data(&state_update, account_id, *data_id)
                    .expect("Genesis storage error")
                    .is_none()
                {
                    pending_data_count += 1;
                    set(
                        &mut state_update,
                        TrieKey::PostponedReceiptId {
                            receiver_id: account_id.clone(),
                            data_id: *data_id,
                        },
                        &receipt.receipt_id,
                    )
                }
            }
            if pending_data_count == 0 {
                panic!("Postponed receipt should have pending data")
            } else {
                set(
                    &mut state_update,
                    TrieKey::PendingDataCount {
                        receiver_id: account_id.clone(),
                        receipt_id: receipt.receipt_id,
                    },
                    &pending_data_count,
                );
                set_postponed_receipt(&mut state_update, &receipt);
            }
        }
        Self::commit(state_update, current_state_root, tries);
    }

    fn apply_delayed_receipts(
        delayed_receipts_indices: DelayedReceiptIndices,
        current_state_root: &mut StateRoot,
        tries: &mut ShardTries,
    ) {
        let mut state_update = tries.new_trie_update( *current_state_root);

        if delayed_receipts_indices != DelayedReceiptIndices::default() {
            set(&mut state_update, TrieKey::DelayedReceiptIndices, &delayed_receipts_indices);
            Self::commit(state_update, current_state_root, tries);
        }
    }

    pub fn apply(
        mut tries: ShardTries,
        config: &RuntimeConfig,
        genesis: &Genesis,
        account_ids: HashSet<AccountId>,
    ) -> StateRoot {
        let mut current_state_root = MerkleHash::default();
        let mut delayed_receipts_indices = DelayedReceiptIndices::default();
        for batch_account_ids in
            account_ids.into_iter().collect::<Vec<AccountId>>().chunks(300_000)
        {
            Self::apply_batch(
                &mut current_state_root,
                &mut delayed_receipts_indices,
                &mut tries,
                config,
                genesis,
                HashSet::from_iter(batch_account_ids),
            );
        }
        Self::apply_delayed_receipts(
            delayed_receipts_indices,
            &mut current_state_root,
            &mut tries,
        );
        current_state_root
    }
}
