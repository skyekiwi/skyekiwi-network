use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Registry;

const PUBLIC_KEY: &str = "38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f";
fn decode_hex_uncompressed(s: &str) -> Vec<u8> {
	(0..s.len())
		.step_by(1)
		.map(|i| u8::from_str_radix(&s[i..i + 1], 16).unwrap())
		.collect()
}

benchmarks! {
	register_secret_keeper {
		let caller: T::AccountId = whitelisted_caller();
		let public_key = decode_hex_uncompressed(PUBLIC_KEY);
	}: register_secret_keeper(RawOrigin::Signed(caller.clone()), public_key, vec![0, 0, 0, 0, 0, 0])
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();

		assert_eq! (all_secret_keepers.len(), 1);
		assert_eq! (all_secret_keepers[0], caller);
	}

	renew_registration {
		let caller: T::AccountId = whitelisted_caller();
		let public_key = decode_hex_uncompressed(PUBLIC_KEY);
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), public_key.clone(), vec![0, 0, 0, 0, 0, 0])?;
	}: renew_registration(RawOrigin::Signed(caller.clone()), public_key, vec![0, 0, 0, 0, 0, 0])
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();

		assert_eq! (all_secret_keepers.len(), 1);
		assert_eq! (all_secret_keepers[0], caller);
	}

	remove_registration {
		let caller: T::AccountId = whitelisted_caller();
		let public_key = decode_hex_uncompressed(PUBLIC_KEY);
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), public_key, vec![0, 0, 0, 0, 0, 0])?;
	}: remove_registration(RawOrigin::Signed(caller.clone()))
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();
		assert_eq! (all_secret_keepers.len(), 0);
	}

	register_running_shard {
		let caller: T::AccountId = whitelisted_caller();
		let public_key = decode_hex_uncompressed(PUBLIC_KEY);
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), public_key, vec![0, 0, 0, 0, 0, 0])?;
	}: register_running_shard(RawOrigin::Signed(caller.clone()), 0)
	verify { }
}

impl_benchmark_test_suite!(
	Registry,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
