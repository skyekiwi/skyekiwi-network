use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use num_traits::pow::Pow;

use skw_vm_primitives::errors::IntegerOverflowError;
pub use skw_vm_primitives::num_rational::Rational;
pub use skw_vm_primitives::config::RuntimeConfig;
use skw_vm_primitives::fees::{transfer_exec_fee, transfer_send_fee, RuntimeFeesConfig};
use skw_vm_primitives::transaction::{
    Action, DeployContractAction, FunctionCallAction, Transaction,
};
use skw_vm_primitives::contract_runtime::{AccountId, Balance, Gas};

/// Describes the cost of converting this transaction into a receipt.
#[derive(Debug)]
pub struct TransactionCost {
    /// Total amount of gas burnt for converting this transaction into a receipt.
    pub gas_burnt: Gas,
    /// The remaining amount of gas used for converting this transaction into a receipt.
    /// It includes gas that is not yet spent, e.g. prepaid gas for function calls and
    /// future execution fees.
    pub gas_remaining: Gas,
    /// The gas price at which the gas was purchased in the receipt.
    pub receipt_gas_price: Balance,
    /// Total costs in tokens for this transaction (including all deposits).
    pub total_cost: Balance,
    /// The amount of tokens burnt by converting this transaction to a receipt.
    pub burnt_amount: Balance,
}

/// Multiplies `gas_price` by the power of `inflation_base` with exponent `inflation_exponent`.
pub fn safe_gas_price_inflated(
    gas_price: Balance,
    inflation_base: Rational,
    inflation_exponent: u8,
) -> Result<Balance, IntegerOverflowError> {
    let numer = BigUint::from(*inflation_base.numer() as usize).pow(inflation_exponent as u32);
    let denom = BigUint::from(*inflation_base.denom() as usize).pow(inflation_exponent as u32);
    // Rounding up
    let inflated_gas_price: BigUint = (numer * gas_price + &denom - 1u8) / denom;
    inflated_gas_price.to_u128().ok_or_else(|| IntegerOverflowError {})
}

pub fn safe_gas_to_balance(gas_price: Balance, gas: Gas) -> Result<Balance, IntegerOverflowError> {
    gas_price.checked_mul(Balance::from(gas)).ok_or_else(|| IntegerOverflowError {})
}

pub fn safe_add_gas(a: Gas, b: Gas) -> Result<Gas, IntegerOverflowError> {
    a.checked_add(b).ok_or_else(|| IntegerOverflowError {})
}

pub fn safe_add_balance(a: Balance, b: Balance) -> Result<Balance, IntegerOverflowError> {
    a.checked_add(b).ok_or_else(|| IntegerOverflowError {})
}

#[macro_export]
macro_rules! safe_add_balance_apply {
    ($x: expr) => {$x};
    ($x: expr, $($rest: expr),+) => {
        safe_add_balance($x, safe_add_balance_apply!($($rest),+))?
    }
}

/// Total sum of gas that needs to be burnt to send these actions.
pub fn total_send_fees(
    config: &RuntimeFeesConfig,
    sender_is_receiver: bool,
    actions: &[Action],
    _receiver_id: &AccountId,
) -> Result<Gas, IntegerOverflowError> {
    let cfg = &config.action_creation_config;
    let mut result = 0;

    for action in actions {
        use Action::*;
        let delta = match action {
            CreateAccount(_) => cfg.create_account_cost.send_fee(sender_is_receiver),
            DeployContract(DeployContractAction { code }) => {
                let num_bytes = code.len() as u64;
                cfg.deploy_contract_cost.send_fee(sender_is_receiver)
                    + cfg.deploy_contract_cost_per_byte.send_fee(sender_is_receiver) * num_bytes
            }
            FunctionCall(FunctionCallAction { method_name, args, .. }) => {
                let num_bytes = method_name.as_bytes().len() as u64 + args.len() as u64;
                cfg.function_call_cost.send_fee(sender_is_receiver)
                    + cfg.function_call_cost_per_byte.send_fee(sender_is_receiver) * num_bytes
            }
            Transfer(_) => {
                transfer_send_fee(cfg, sender_is_receiver)
            },
            DeleteAccount(_) => cfg.delete_account_cost.send_fee(sender_is_receiver),
        };
        result = safe_add_gas(result, delta)?;
    }
    Ok(result)
}

pub fn exec_fee(
    config: &RuntimeFeesConfig,
    action: &Action,
    _receiver_id: &AccountId,
) -> Gas {
    let cfg = &config.action_creation_config;
    use Action::*;

    match action {
        CreateAccount(_) => cfg.create_account_cost.exec_fee(),
        DeployContract(DeployContractAction { code }) => {
            let num_bytes = code.len() as u64;
            cfg.deploy_contract_cost.exec_fee()
                + cfg.deploy_contract_cost_per_byte.exec_fee() * num_bytes
        }
        FunctionCall(FunctionCallAction { method_name, args, .. }) => {
            let num_bytes = method_name.as_bytes().len() as u64 + args.len() as u64;
            cfg.function_call_cost.exec_fee()
                + cfg.function_call_cost_per_byte.exec_fee() * num_bytes
        },
        Transfer(_) => {
            transfer_exec_fee(cfg)
        },
        DeleteAccount(_) => cfg.delete_account_cost.exec_fee(),
    }
}

/// Returns transaction costs for a given transaction.
pub fn tx_cost(
    config: &RuntimeFeesConfig,
    transaction: &Transaction,
    gas_price: Balance,
    sender_is_receiver: bool,
) -> Result<TransactionCost, IntegerOverflowError> {
    let mut gas_burnt: Gas = config.action_receipt_creation_config.send_fee(sender_is_receiver);
    gas_burnt = safe_add_gas(
        gas_burnt,
        total_send_fees(
            config,
            sender_is_receiver,
            &transaction.actions,
            &transaction.receiver_id,
        )?,
    )?;
    let prepaid_gas = total_prepaid_gas(&transaction.actions)?;
    // If signer is equals to receiver the receipt will be processed at the same block as this
    // transaction. Otherwise it will processed in the next block and the gas might be inflated.
    let initial_receipt_hop = if transaction.signer_id == transaction.receiver_id { 0 } else { 1 };
    let minimum_new_receipt_gas = config.min_receipt_with_function_call_gas();
    // In case the config is free, we don't care about the maximum depth.
    let receipt_gas_price = if gas_price == 0 {
        0
    } else {
        let maximum_depth =
            if minimum_new_receipt_gas > 0 { prepaid_gas / minimum_new_receipt_gas } else { 0 };
        let inflation_exponent = u8::try_from(initial_receipt_hop + maximum_depth)
            .map_err(|_| IntegerOverflowError {})?;
        safe_gas_price_inflated(
            gas_price,
            config.pessimistic_gas_price_inflation_ratio,
            inflation_exponent,
        )?
    };

    let mut gas_remaining =
        safe_add_gas(prepaid_gas, config.action_receipt_creation_config.exec_fee())?;
    gas_remaining = safe_add_gas(
        gas_remaining,
        total_prepaid_exec_fees(
            config,
            &transaction.actions,
            &transaction.receiver_id,
        )?,
    )?;
    let burnt_amount = safe_gas_to_balance(gas_price, gas_burnt)?;
    let remaining_gas_amount = safe_gas_to_balance(receipt_gas_price, gas_remaining)?;
    let mut total_cost = safe_add_balance(burnt_amount, remaining_gas_amount)?;
    total_cost = safe_add_balance(total_cost, total_deposit(&transaction.actions)?)?;
    Ok(TransactionCost { gas_burnt, gas_remaining, receipt_gas_price, total_cost, burnt_amount })
}

/// Total sum of gas that would need to be burnt before we start executing the given actions.
pub fn total_prepaid_exec_fees(
    config: &RuntimeFeesConfig,
    actions: &[Action],
    receiver_id: &AccountId,
) -> Result<Gas, IntegerOverflowError> {
    let mut result = 0;
    for action in actions {
        let delta = exec_fee(config, action, receiver_id);
        result = safe_add_gas(result, delta)?;
    }
    Ok(result)
}

/// Get the total sum of deposits for given actions.
pub fn total_deposit(actions: &[Action]) -> Result<Balance, IntegerOverflowError> {
    let mut total_balance: Balance = 0;
    for action in actions {
        total_balance = safe_add_balance(total_balance, action.get_deposit_balance())?;
    }
    Ok(total_balance)
}

/// Get the total sum of prepaid gas for given actions.
pub fn total_prepaid_gas(actions: &[Action]) -> Result<Gas, IntegerOverflowError> {
    actions.iter().try_fold(0, |acc, action| safe_add_gas(acc, action.get_prepaid_gas()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_gas_price_inflated() {
        assert_eq!(safe_gas_price_inflated(10000, Rational::new(101, 100), 1).unwrap(), 10100);
        assert_eq!(safe_gas_price_inflated(10000, Rational::new(101, 100), 2).unwrap(), 10201);
        // Rounded up
        assert_eq!(safe_gas_price_inflated(10000, Rational::new(101, 100), 3).unwrap(), 10304);
        assert_eq!(safe_gas_price_inflated(10000, Rational::new(101, 100), 32).unwrap(), 13750);
    }
}
