pub mod outcome;
#[doc(inline)]
pub use outcome::*;
// mod cache;

pub mod runtime;
pub mod units;
pub mod user;

// #[doc(hidden)]
// pub use skw_vm_primitives::*;

#[doc(inline)]
pub use units::*;
#[doc(inline)]
pub use user::*;

#[doc(hidden)]
pub use lazy_static_include;

use skw_vm_primitives::account_id::AccountId as PAccountId;
use std::convert::TryFrom;
pub(crate) fn new_p_account(account_id :&str) -> PAccountId {
    PAccountId::try_from(account_id.to_string()).unwrap()
}
