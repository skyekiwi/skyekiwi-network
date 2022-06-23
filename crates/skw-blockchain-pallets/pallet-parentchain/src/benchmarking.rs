use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};

#[allow(unused)]
use crate::Pallet as Parentchain;
use pallet_registry::Pallet as Registry;
use sp_std::vec::Vec;
use skw_blockchain_primitives::types::CallIndex;

const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

benchmarks! {
	set_shard_confirmation_threshold {
		let s = 0u32;
		let caller: T::AccountId = whitelisted_caller();
	}: set_shard_confirmation_threshold(RawOrigin::Root, s, 2)
	verify {
		assert_eq!(ShardConfirmationThreshold::<T>::get(s), Some(2u64));
	}

	submit_outcome {
		let s in 1 .. 100;

		let shard_id = 0u32;
		let caller: T::AccountId = whitelisted_caller();
		let now = frame_system::Pallet::<T>::block_number();

		let state_root = [0u8; 32];

		let mut outcome_call_index: Vec<CallIndex> = Vec::new();
		let mut outcome: Vec<Vec<u8>> = Vec::new();

		for i in 0 .. s {
			outcome_call_index.push(i as CallIndex);
			outcome.push([i as u8; 50_000].to_vec());
		}
		
		Registry::<T>::register_secret_keeper( RawOrigin::Signed(caller.clone()).into(), PUBLIC_KEY.to_vec(), Vec::new() )?;
		Registry::<T>::register_running_shard( RawOrigin::Signed(caller.clone()).into(), 0 )?;
		Parentchain::<T>::set_shard_confirmation_threshold( RawOrigin::Root.into(), shard_id, 1 )?;
	}: submit_outcome(RawOrigin::Signed(caller), now, shard_id, state_root, outcome_call_index, outcome)
	verify {
		assert_eq!(Confirmation::<T>::get(shard_id, now), Some(1u64));
	}
}

impl_benchmark_test_suite!(
	Parentchain,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
