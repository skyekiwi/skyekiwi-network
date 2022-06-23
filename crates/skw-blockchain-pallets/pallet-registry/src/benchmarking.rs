use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};

#[allow(unused)]
use crate::Pallet as Registry;
use sp_std::vec;

const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

benchmarks! {
	register_secret_keeper {
		let caller: T::AccountId = whitelisted_caller();
	}: register_secret_keeper(RawOrigin::Signed(caller.clone()), PUBLIC_KEY.to_vec(), vec![0, 0, 0, 0, 0, 0])
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();

		assert_eq! (all_secret_keepers.len(), 1);
		assert_eq! (all_secret_keepers[0], caller);
	}

	renew_registration {
		let caller: T::AccountId = whitelisted_caller();
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), PUBLIC_KEY.to_vec(), vec![0, 0, 0, 0, 0, 0])?;
	}: renew_registration(RawOrigin::Signed(caller.clone()), PUBLIC_KEY.to_vec(), vec![0, 0, 0, 0, 0, 0])
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();

		assert_eq! (all_secret_keepers.len(), 1);
		assert_eq! (all_secret_keepers[0], caller);
	}

	remove_registration {
		let caller: T::AccountId = whitelisted_caller();
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), PUBLIC_KEY.to_vec(), vec![0, 0, 0, 0, 0, 0])?;
	}: remove_registration(RawOrigin::Signed(caller.clone()))
	verify {
		let all_secret_keepers = Registry::<T>::secret_keepers().unwrap();
		assert_eq! (all_secret_keepers.len(), 0);
	}

	register_running_shard {
		let caller: T::AccountId = whitelisted_caller();
		Registry::<T>::register_secret_keeper(RawOrigin::Signed(caller.clone()).into(), PUBLIC_KEY.to_vec(), vec![0, 0, 0, 0, 0, 0])?;
	}: register_running_shard(RawOrigin::Signed(caller.clone()), 0)
	verify { }

	register_user_public_key {
		let caller: T::AccountId = whitelisted_caller();
	}: register_user_public_key(RawOrigin::Signed(caller.clone()), PUBLIC_KEY.to_vec())
	verify { }
}

impl_benchmark_test_suite!(
	Registry,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
