pub mod outcome;
#[doc(inline)]
pub use outcome::*;
// mod cache;

pub mod runtime;
pub mod units;
pub mod user;
#[doc(hidden)]
pub use skw_vm_primitives::*;
#[doc(inline)]
pub use units::*;
#[doc(inline)]
pub use user::*;

#[doc(hidden)]
pub use lazy_static_include;
use skw_vm_primitives::account_id::AccountId;

pub fn root_account() -> AccountId {
    "root".parse().unwrap()
}
pub fn alice_account() -> AccountId {
    "alice".parse().unwrap()
}
pub fn bob_account() -> AccountId {
    "bob".parse().unwrap()
}
pub fn caller_account() -> AccountId {
    "charlie".parse().unwrap()
}
pub fn contract_account() -> AccountId {
    "status".parse().unwrap()
}