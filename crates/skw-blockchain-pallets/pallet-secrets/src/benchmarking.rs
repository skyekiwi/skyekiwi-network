use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as Secrets;
use sp_std::vec::Vec;

fn sized_preimage<T: Config>(size: u32) -> Vec<u8> {
	let mut preimage = Vec::new();
	preimage.resize(size as usize, 0);
	preimage
}

benchmarks! {
	register_secret {
		let s in 0 .. 4194304; 
		let preimage = sized_preimage::<T>(s);
		let caller: T::AccountId = whitelisted_caller();
	}: register_secret(RawOrigin::Signed(caller), preimage)
	verify {
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		// assert_eq! (Secrets::<T>::metadata_of(secret_id), Some(T::Hashing::hash(&preimage[..])));
	}

	nominate_member {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;

		let caller2: T::AccountId = whitelisted_caller();
	}: nominate_member(RawOrigin::Signed(caller), secret_id, caller2)
	verify {	}

	remove_member {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;

		let caller2: T::AccountId = whitelisted_caller();
		Secrets::<T>::nominate_member(RawOrigin::Signed(caller.clone()).into(), secret_id, caller2.clone())?;
	}: remove_member(RawOrigin::Signed(caller), secret_id, caller2)
	verify { }

	force_nominate_member {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
	}: force_nominate_member(RawOrigin::Root, secret_id, caller2)
	verify { }

	force_remove_member {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
		Secrets::<T>::nominate_member(RawOrigin::Signed(caller.clone()).into(), secret_id, caller2.clone())?;
	}: force_remove_member(RawOrigin::Root, secret_id, caller2)
	verify { }

	force_change_owner {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
	}: force_change_owner(RawOrigin::Root, secret_id, caller2)
	verify { }

	update_metadata {
		let s in 0 .. 4194304;
		let preimage = sized_preimage::<T>(s);

		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;

		let secret_id = Secrets::<T>::current_secret_id() - 1;
	}: update_metadata(RawOrigin::Signed(caller), secret_id, preimage) 
	verify { 
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		// assert_eq! (Secrets::<T>::metadata_of(secret_id), Some(hash));
	}

	burn_secret {
		const METADATA1: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), METADATA1[..].to_vec())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
	}: burn_secret(RawOrigin::Signed(caller), secret_id)
	verify { }
}

impl_benchmark_test_suite!(
	Secrets,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
