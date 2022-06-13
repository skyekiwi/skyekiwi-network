use super::*;
use sp_std::vec::Vec;
use frame_support::pallet_prelude::Encode;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
use skw_blockchain_primitives::types::PublicKey;
use frame_support::traits::Currency;
use frame_support::sp_runtime::traits::Bounded;
const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const PUBLIC_KEY: PublicKey = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

benchmarks! {
	force_create_enclave_account {
		let s = 0u32;
		let caller: T::AccountId = whitelisted_caller();		
		pallet_s_contract::Pallet::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), s, caller.clone())?;
		
		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&pallet_s_contract::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
			receipt_public_key: T::AccountId::encode(&caller.clone()).try_into().unwrap(),
			encrypted_egress: false,
			transaction_action: 0, 
			amount: Some(10),
			wasm_blob_path: None,
			method: None,  
			args: None,
		});

		let calls = skw_blockchain_primitives::types::Calls {
			ops: all_calls,
			block_number: Some(1),
			shard_id: 0
		};
		let encoded_calls = skw_blockchain_primitives::BorshSerialize::try_to_vec(&calls).unwrap();
		pallet_s_contract::Pallet::<T>::initialize_shard(
			RawOrigin::Signed(caller.clone()).into(), 0, encoded_calls.clone(),
			IPFS_CID_1.as_bytes().to_vec(),
			PUBLIC_KEY
		)?;
	}: force_create_enclave_account(RawOrigin::Root, s, caller)
	verify { }

	create_account {
		let s = 0u32;
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
		pallet_s_contract::Pallet::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), 0, caller.clone())?;
		
		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&pallet_s_contract::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
			receipt_public_key: T::AccountId::encode(&caller.clone()).try_into().unwrap(),
			encrypted_egress: false,
			transaction_action: 0, 
			amount: Some(10),
			wasm_blob_path: None,
			method: None,  
			args: None,
		});

		let calls = skw_blockchain_primitives::types::Calls {
			ops: all_calls,
			block_number: Some(1),
			shard_id: 0
		};
		let encoded_calls = skw_blockchain_primitives::BorshSerialize::try_to_vec(&calls).unwrap();
		pallet_s_contract::Pallet::<T>::initialize_shard(
			RawOrigin::Signed(caller.clone()).into(), s, encoded_calls.clone(),
			IPFS_CID_1.as_bytes().to_vec(),
			PUBLIC_KEY
		)?;
	}: create_account(RawOrigin::Signed(caller), s)
	verify { }
}

impl_benchmark_test_suite!(
	Parentchain,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
