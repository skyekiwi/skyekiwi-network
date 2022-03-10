//! Benchmarking setup for pallet-template

use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Secrets;

// SBP M1 review: missing benchmarks

benchmarks! {
	register_secret {
		let s in 0 .. 100;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller), s)
	verify {
		assert_eq!(Metadata::<T>::get(), Some(s));
	}
}

impl_benchmark_test_suite!(
	Secrets,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
