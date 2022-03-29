use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Parentchain;

benchmarks! {
	set_shard_confirmation_threshold {
		let s = 0u64;
		let caller: T::AccountId = whitelisted_caller();
	}: set_shard_confirmation_threshold(RawOrigin::Root, s, 2)
	verify {
		assert_eq!(ShardConfirmationThreshold::<T>::get(s), Some(2u64));
	}
}

impl_benchmark_test_suite!(
	Parentchain,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
