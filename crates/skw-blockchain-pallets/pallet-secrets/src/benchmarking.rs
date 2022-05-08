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
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		assert_eq! (Metadata::<T>::get(secret_id), Some(IPFS_CID.into()));
	}

	nominate_member {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;

		let caller2: T::AccountId = whitelisted_caller();
	}: nominate_member(RawOrigin::Signed(caller), secret_id, caller2)
	verify {	}

	remove_member {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;

		let caller2: T::AccountId = whitelisted_caller();
		Secrets::<T>::nominate_member(RawOrigin::Signed(caller.clone()).into(), secret_id, caller2.clone())?;
	}: remove_member(RawOrigin::Signed(caller), secret_id, caller2)
	verify { }

	force_nominate_member {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
	}: force_nominate_member(RawOrigin::Root, secret_id, caller2)
	verify { }

	force_remove_member {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
		Secrets::<T>::nominate_member(RawOrigin::Signed(caller.clone()).into(), secret_id, caller2.clone())?;
	}: force_remove_member(RawOrigin::Root, secret_id, caller2)
	verify { }

	force_change_owner {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
		let caller2: T::AccountId = whitelisted_caller();
	}: force_change_owner(RawOrigin::Root, secret_id, caller2)
	verify { }

	update_metadata {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		const IPFS_CID2: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi89";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
	}: update_metadata(RawOrigin::Signed(caller), secret_id, IPFS_CID2.into())
	verify { }

	burn_secret {
		const IPFS_CID: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
		let caller: T::AccountId = whitelisted_caller();
		Secrets::<T>::register_secret(RawOrigin::Signed(caller.clone()).into(), IPFS_CID.into())?;
		let secret_id = Secrets::<T>::current_secret_id() - 1;
	}: burn_secret(RawOrigin::Signed(caller), secret_id)
	verify { }
}

impl_benchmark_test_suite!(
	Secrets,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
