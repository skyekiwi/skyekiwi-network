use crate::safe_add_balance_apply;

use crate::config::{
    safe_add_balance, safe_add_gas, safe_gas_to_balance, total_deposit, total_prepaid_exec_fees, total_prepaid_gas,
};
use crate::{ApplyStats, DelayedReceiptIndices};
use skw_vm_primitives::errors::{
    BalanceMismatchError, IntegerOverflowError, RuntimeError, StorageError,
};
use skw_vm_primitives::receipt::{Receipt, ReceiptEnum};
use skw_vm_primitives::fees::RuntimeFeesConfig;
use skw_vm_primitives::transaction::SignedTransaction;
use skw_vm_primitives::trie_key::TrieKey;
use skw_vm_primitives::contract_runtime::{AccountId, Balance};

use skw_vm_store::{get, get_account, get_postponed_receipt, TrieUpdate};
use std::collections::HashSet;

pub(crate) fn check_balance(
    transaction_costs: &RuntimeFeesConfig,
    initial_state: &TrieUpdate,
    final_state: &TrieUpdate,
    incoming_receipts: &[Receipt],
    transactions: &[SignedTransaction],
    outgoing_receipts: &[Receipt],
    stats: &ApplyStats,
) -> Result<(), RuntimeError> {
    // Delayed receipts
    let initial_delayed_receipt_indices: DelayedReceiptIndices =
        get(initial_state, &TrieKey::DelayedReceiptIndices)?.unwrap_or_default();
    let final_delayed_receipt_indices: DelayedReceiptIndices =
        get(final_state, &TrieKey::DelayedReceiptIndices)?.unwrap_or_default();
    let get_delayed_receipts = |from_index, to_index, state| {
        (from_index..to_index)
            .map(|index| {
                get(state, &TrieKey::DelayedReceipt { index })?.ok_or_else(|| {
                    StorageError::StorageInconsistentState(format!(
                        "Delayed receipt #{} should be in the state",
                        index
                    ))
                })
            })
            .collect::<Result<Vec<Receipt>, StorageError>>()
    };

    // Previously delayed receipts that were processed this time.
    let processed_delayed_receipts = get_delayed_receipts(
        initial_delayed_receipt_indices.first_index,
        final_delayed_receipt_indices.first_index,
        initial_state,
    )?;
    // Receipts that were not processed this time and are delayed now.
    let new_delayed_receipts = get_delayed_receipts(
        initial_delayed_receipt_indices.next_available_index,
        final_delayed_receipt_indices.next_available_index,
        final_state,
    )?;

    // Accounts
    let all_accounts_ids: HashSet<AccountId> = transactions
        .iter()
        .map(|tx| tx.transaction.signer_id.clone())
        .chain(incoming_receipts.iter().map(|r| r.receiver_id.clone()))
        .chain(processed_delayed_receipts.iter().map(|r| r.receiver_id.clone()))
        .collect();

    let total_accounts_balance = |state| -> Result<Balance, RuntimeError> {
        Ok(all_accounts_ids
            .iter()
            .map(|account_id| {
                get_account(state, account_id)?.map_or(Ok(0), |a| {
                    safe_add_balance(a.amount(), a.locked())
                        .map_err(|_| RuntimeError::UnexpectedIntegerOverflow)
                })
            })
            .collect::<Result<Vec<Balance>, RuntimeError>>()?
            .into_iter()
            .try_fold(0u128, safe_add_balance)?)
    };
    let initial_accounts_balance = total_accounts_balance(initial_state)?;
    let final_accounts_balance = total_accounts_balance(final_state)?;
    // Receipts
    let receipt_cost = |receipt: &Receipt| -> Result<Balance, IntegerOverflowError> {
        Ok(match &receipt.receipt {
            ReceiptEnum::Action(action_receipt) => {
                let mut total_cost = total_deposit(&action_receipt.actions)?;
                if !AccountId::is_system(&receipt.predecessor_id) {
                    let mut total_gas = safe_add_gas(
                        transaction_costs.action_receipt_creation_config.exec_fee(),
                        total_prepaid_exec_fees(
                            transaction_costs,
                            &action_receipt.actions,
                            &receipt.receiver_id,
                        )?,
                    )?;
                    total_gas =
                        safe_add_gas(total_gas, total_prepaid_gas(&action_receipt.actions)?)?;
                    let total_gas_cost = safe_gas_to_balance(action_receipt.gas_price, total_gas)?;
                    total_cost = safe_add_balance(total_cost, total_gas_cost)?;
                }
                total_cost
            }
            ReceiptEnum::Data(_) => 0,
        })
    };
    let receipts_cost = |receipts: &[Receipt]| -> Result<Balance, IntegerOverflowError> {
        receipts
            .iter()
            .map(receipt_cost)
            .collect::<Result<Vec<Balance>, IntegerOverflowError>>()?
            .into_iter()
            .try_fold(0u128, safe_add_balance)
    };

    let incoming_receipts_balance = receipts_cost(incoming_receipts)?;
    let outgoing_receipts_balance = receipts_cost(outgoing_receipts)?;
    let processed_delayed_receipts_balance = receipts_cost(&processed_delayed_receipts)?;
    let new_delayed_receipts_balance = receipts_cost(&new_delayed_receipts)?;

    // Postponed actions receipts. The receipts can be postponed and stored with the receiver's
    // account ID when the input data is not received yet.
    // We calculate all potential receipts IDs that might be postponed initially or after the
    // execution.
    let all_potential_postponed_receipt_ids = incoming_receipts
        .iter()
        .chain(processed_delayed_receipts.iter())
        .map(|receipt| {
            let account_id = &receipt.receiver_id;
            match &receipt.receipt {
                ReceiptEnum::Action(_) => Ok(Some((account_id.clone(), receipt.receipt_id))),
                ReceiptEnum::Data(data_receipt) => {
                    if let Some(receipt_id) = get(
                        initial_state,
                        &TrieKey::PostponedReceiptId {
                            receiver_id: account_id.clone(),
                            data_id: data_receipt.data_id,
                        },
                    )? {
                        Ok(Some((account_id.clone(), receipt_id)))
                    } else {
                        Ok(None)
                    }
                }
            }
        })
        .collect::<Result<Vec<Option<_>>, StorageError>>()?
        .into_iter()
        .filter_map(|x| x)
        .collect::<HashSet<_>>();

    let total_postponed_receipts_cost = |state| -> Result<Balance, RuntimeError> {
        Ok(all_potential_postponed_receipt_ids
            .iter()
            .map(|(account_id, receipt_id)| {
                Ok(get_postponed_receipt(state, account_id, *receipt_id)?
                    .map_or(Ok(0), |r| receipt_cost(&r))?)
            })
            .collect::<Result<Vec<Balance>, RuntimeError>>()?
            .into_iter()
            .try_fold(0u128, safe_add_balance)?)
    };
    let initial_postponed_receipts_balance = total_postponed_receipts_cost(initial_state)?;
    let final_postponed_receipts_balance = total_postponed_receipts_cost(final_state)?;
    // Sum it up

    let initial_balance = safe_add_balance_apply!(
        initial_accounts_balance,
        incoming_receipts_balance,
        processed_delayed_receipts_balance,
        initial_postponed_receipts_balance
    );
    let final_balance = safe_add_balance_apply!(
        final_accounts_balance,
        outgoing_receipts_balance,
        new_delayed_receipts_balance,
        final_postponed_receipts_balance,
        stats.tx_burnt_amount,
        stats.slashed_burnt_amount,
        stats.other_burnt_amount
    );
    if initial_balance != final_balance {
        Err(BalanceMismatchError {
            // Inputs
            initial_accounts_balance,
            incoming_receipts_balance,
            processed_delayed_receipts_balance,
            initial_postponed_receipts_balance,
            // Outputs
            final_accounts_balance,
            outgoing_receipts_balance,
            new_delayed_receipts_balance,
            final_postponed_receipts_balance,
            tx_burnt_amount: stats.tx_burnt_amount,
            slashed_burnt_amount: stats.slashed_burnt_amount,
            other_burnt_amount: stats.other_burnt_amount,
        }
        .into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApplyStats;
    use skw_vm_primitives::crypto::{InMemorySigner, KeyType};

    use skw_vm_primitives::contract_runtime::{
        hash_bytes, CryptoHash, MerkleHash
    };
    use skw_vm_primitives::state::{StateChangeCause};

    use skw_vm_primitives::receipt::ActionReceipt;
    use skw_vm_primitives::fees::RuntimeFeesConfig;
    use skw_vm_primitives::test_utils::account_new;
    use skw_vm_primitives::transaction::{Action, TransferAction};

    use skw_vm_store::set_account;
    use skw_vm_store::test_utils::create_tries;

    pub fn alice_account() -> AccountId {
        AccountId::system()
    }
    pub fn bob_account() -> AccountId {
        AccountId::test2()
    }

    use assert_matches::assert_matches;

    /// Initial balance used in tests.
    pub const TESTING_INIT_BALANCE: Balance = 1_000_000_000 * NEAR_BASE;

    /// One NEAR, divisible by 10^24.
    pub const NEAR_BASE: Balance = 1_000_000_000_000_000_000_000_000;

    #[test]
    fn test_check_balance_no_op() {
        let tries = create_tries();
        let root = MerkleHash::default();
        let initial_state = tries.new_trie_update(root);
        let final_state = tries.new_trie_update(root);
        let transaction_costs = RuntimeFeesConfig::test();
        check_balance(
            &transaction_costs,
            &initial_state,
            &final_state,
            &[],
            &[],
            &[],
            &ApplyStats::default(),
        )
        .unwrap();
    }

    #[test]
    fn test_check_balance_unaccounted_refund() {
        let tries = create_tries();
        let root = MerkleHash::default();
        let initial_state = tries.new_trie_update(root);
        let final_state = tries.new_trie_update(root);
        let transaction_costs = RuntimeFeesConfig::test();
        let err = check_balance(
            &transaction_costs,
            &initial_state,
            &final_state,
            &[Receipt::new_force_transfer(&alice_account(), 1000)],
            &[],
            &[],
            &ApplyStats::default(),
        )
        .unwrap_err();
        assert_matches!(err, RuntimeError::BalanceMismatchError(_));
    }

    #[test]
    fn test_check_force_transfer() {
        let tries = create_tries();
        let root = MerkleHash::default();
        let account_id = alice_account();

        let initial_balance = TESTING_INIT_BALANCE;
        let refund_balance = 1000;

        let mut initial_state = tries.new_trie_update(root);
        let initial_account = account_new(initial_balance, hash_bytes(&[]), 0);
        set_account(&mut initial_state, account_id.clone(), &initial_account);
        initial_state.commit(StateChangeCause::NotWritableToDisk);

        let mut final_state = tries.new_trie_update(root);
        let final_account = account_new(initial_balance + refund_balance, hash_bytes(&[]), 0);
        set_account(&mut final_state, account_id.clone(), &final_account);
        final_state.commit(StateChangeCause::NotWritableToDisk);

        let transaction_costs = RuntimeFeesConfig::test();
        check_balance(
            &transaction_costs,
            &initial_state,
            &final_state,
            &[Receipt::new_force_transfer(&account_id, refund_balance)],
            &[],
            &[],
            &ApplyStats::default(),
        ).unwrap();
    }

    #[test]
    fn test_check_balance_tx_to_receipt() {
        let tries = create_tries();
        let root = MerkleHash::default();
        let account_id = alice_account();

        let initial_balance = TESTING_INIT_BALANCE / 2;
        let deposit = 500_000_000;
        let gas_price = 100;
        let cfg = RuntimeFeesConfig::test();
        let exec_gas = cfg.action_receipt_creation_config.exec_fee()
            + cfg.action_creation_config.transfer_cost.exec_fee();
        let send_gas = cfg.action_receipt_creation_config.send_fee(false)
            + cfg.action_creation_config.transfer_cost.send_fee(false);

        let total_validator_reward = send_gas as Balance * gas_price;
        let mut initial_state = tries.new_trie_update(root);
        let initial_account = account_new(initial_balance, hash_bytes(&[]), 0);
        set_account(&mut initial_state, account_id.clone(), &initial_account);
        initial_state.commit(StateChangeCause::NotWritableToDisk);

        let mut final_state = tries.new_trie_update(root);
        let final_account = account_new(
            initial_balance - (exec_gas /* + send_gas send_gas is free because root */ ) as Balance * gas_price - deposit,
            hash_bytes(&[]),
            0
        );
        set_account(&mut final_state, account_id.clone(), &final_account);
        final_state.commit(StateChangeCause::NotWritableToDisk);

        let signer =
            InMemorySigner::from_seed(KeyType::ED25519, &[0; 32][..]);
        let tx = SignedTransaction::send_money(
            1,
            account_id,
            bob_account(),
            &signer,
            deposit,
            CryptoHash::default(),
        );
        let receipt = Receipt {
            predecessor_id: tx.transaction.signer_id.clone(),
            receiver_id: tx.transaction.receiver_id.clone(),
            receipt_id: Default::default(),
            receipt: ReceiptEnum::Action(ActionReceipt {
                signer_id: tx.transaction.signer_id.clone(),
                gas_price,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![Action::Transfer(TransferAction { deposit })],
            }),
        };

        check_balance(
            &cfg,
            &initial_state,
            &final_state,
            &[],
            &[tx],
            &[receipt],
            &ApplyStats {
                tx_burnt_amount: total_validator_reward,
                other_burnt_amount: 0,
                slashed_burnt_amount: 0,
                gas_deficit_amount: 0,
            },
        )
        .unwrap();
    }

    #[test]
    fn test_total_balance_overflow_returns_unexpected_overflow() {
        let tries = create_tries();
        let root = MerkleHash::default();
        
        let signer1 =
            InMemorySigner::from_seed(KeyType::SR25519, &[0; 32][..]);

        let signer2 =
            InMemorySigner::from_seed(KeyType::SR25519, &[1; 32][..]);

        let alice_id = signer1.account_id();
        let bob_id = signer2.account_id() ;
        
        let gas_price = 100;
        let deposit = 1000;

        let mut initial_state = tries.new_trie_update(root);
        let alice = account_new(u128::MAX, hash_bytes(&[]), 0);
        let bob = account_new(1u128, hash_bytes(&[]), 0);

        set_account(&mut initial_state, alice_id.clone(), &alice);
        set_account(&mut initial_state, bob_id.clone(), &bob);
        initial_state.commit(StateChangeCause::NotWritableToDisk);

        let tx =
            SignedTransaction::send_money(
                0, 
                alice_id, 
                bob_id, 
                &signer1, 
                1, 
                CryptoHash::default()
            );

        let receipt = Receipt {
            predecessor_id: tx.transaction.signer_id.clone(),
            receiver_id: tx.transaction.receiver_id.clone(),
            receipt_id: Default::default(),
            receipt: ReceiptEnum::Action(ActionReceipt {
                signer_id: tx.transaction.signer_id.clone(),
                gas_price,
                output_data_receivers: vec![],
                input_data_ids: vec![],
                actions: vec![Action::Transfer(TransferAction { deposit })],
            }),
        };

        let transaction_costs = RuntimeFeesConfig::test();
        assert_eq!(
            check_balance(
                &transaction_costs,
                &initial_state,
                &initial_state,
                &[receipt],
                &[tx],
                &[],
                &ApplyStats::default(),
            ),
            Err(RuntimeError::UnexpectedIntegerOverflow)
        );
    }
}
