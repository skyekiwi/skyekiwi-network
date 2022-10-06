use crate::test_utils::VMContextBuilder;

use crate::AccountId;
use skw_vm_primitives::config::VMConfig;

pub fn alice() -> AccountId {
    AccountId::test(0)
}

pub fn bob() -> AccountId {
    AccountId::test(1)
}

pub fn carol() -> AccountId {
    AccountId::test(2)
}

/// free == effectively unlimited gas
/// Sets up the blockchain interface with a [`VMConfig`] which sets the gas costs to zero.
pub fn setup_free() {
    crate::testing_env!(VMContextBuilder::new().build(), VMConfig::free())
}
