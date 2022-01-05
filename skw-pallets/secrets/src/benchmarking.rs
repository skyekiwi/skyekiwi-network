//! Benchmarking setup for pallet-template

use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Secrets;

benchmarks! {
	register_secret {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
	}: register_secret(RawOrigin::Signed(caller), IPFS_CID.into())
	verify {
		// assert_eq!(Metadata::<T>::get(), Some(s));
	}
}

impl_benchmark_test_suite!(
	Secrets,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
