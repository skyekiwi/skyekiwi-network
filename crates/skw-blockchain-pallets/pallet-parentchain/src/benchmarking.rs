use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Parentchain;
use pallet_registry::Pallet as Registry;

const PUBLIC_KEY: &str = "38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f";
fn decode_hex_uncompressed(s: &str) -> Vec<u8> {
	(0..s.len())
		.step_by(1)
		.map(|i| u8::from_str_radix(&s[i..i + 1], 16).unwrap())
		.collect()
}

benchmarks! {
	set_shard_confirmation_threshold {
		let s = 0u64;
		let caller: T::AccountId = whitelisted_caller();
	}: set_shard_confirmation_threshold(RawOrigin::Root, s, 2)
	verify {
		assert_eq!(ShardConfirmationThreshold::<T>::get(s), Some(2u64));
	}

	submit_outcome {
		let s in 1 .. 100;

		let shard_id = 0u64;
		let caller: T::AccountId = whitelisted_caller();
		let now = frame_system::Pallet::<T>::block_number();

		let state_root = [0u8; 32];
		let state_file_hash = [0u8; 32];

		let mut outcome_call_index: Vec<CallIndex> = Vec::new();
		let mut outcome: Vec<Vec<u8>> = Vec::new();

		for i in 0 .. s {
			outcome_call_index.push(i as CallIndex);
			outcome.push([i as u8; 50_000].to_vec());
		}
		
		let public_key = decode_hex_uncompressed(PUBLIC_KEY);
		Registry::<T>::register_secret_keeper( RawOrigin::Signed(caller.clone()).into(),  public_key.clone(), Vec::new() )?;
		Registry::<T>::register_running_shard( RawOrigin::Signed(caller.clone()).into(), 0 )?;
		Parentchain::<T>::set_shard_confirmation_threshold( RawOrigin::Root.into(), shard_id, 1 )?;
	}: submit_outcome(RawOrigin::Signed(caller), now, shard_id, state_root, state_file_hash, outcome_call_index, outcome)
	verify {
		assert_eq!(Confirmation::<T>::get(shard_id, now), Some(1u64));
	}
}

impl_benchmark_test_suite!(
	Parentchain,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
