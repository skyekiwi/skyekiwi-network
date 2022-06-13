use super::*;
use sp_std::vec::Vec;
use frame_support::pallet_prelude::Encode;
use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite};
#[allow(unused)]
use crate::Pallet as SContract;
use skw_blockchain_primitives::types::PublicKey;

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const PUBLIC_KEY: PublicKey = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

benchmarks! {
	
	add_authorized_shard_operator {
		let caller: T::AccountId = whitelisted_caller();
	}: add_authorized_shard_operator(RawOrigin::Root, 0, caller)
	verify { }

	initialize_shard {
		let caller: T::AccountId = whitelisted_caller();

		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&crate::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
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
		SContract::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), 0, caller.clone())?;
	}: initialize_shard(RawOrigin::Signed(caller), 0, encoded_calls.clone(),
		IPFS_CID_1.as_bytes().to_vec(),
		PUBLIC_KEY
	) verify { }

	register_contract {
		let caller: T::AccountId = whitelisted_caller();
		SContract::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), 0, caller.clone())?;
		
		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&crate::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
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
		SContract::<T>::initialize_shard(
			RawOrigin::Signed(caller.clone()).into(), 0, encoded_calls.clone(),
			IPFS_CID_1.as_bytes().to_vec(),
			PUBLIC_KEY
		)?;
	}: register_contract (
		RawOrigin::Signed(caller),
		"contract_name".as_bytes().to_vec(),
		IPFS_CID_1.as_bytes().to_vec(),
		encoded_calls.clone(),
		0
	) verify { }

	push_call {
		let caller: T::AccountId = whitelisted_caller();
		SContract::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), 0, caller.clone())?;
		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&crate::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
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
		SContract::<T>::initialize_shard(
			RawOrigin::Signed(caller.clone()).into(), 0, encoded_calls.clone(),
			IPFS_CID_1.as_bytes().to_vec(),
			PUBLIC_KEY
		)?;
	}: push_call( RawOrigin::Signed(caller), 0, encoded_calls.clone() ) verify { }

	shard_rollup {
		let caller: T::AccountId = whitelisted_caller();
		SContract::<T>::add_authorized_shard_operator(RawOrigin::Root.into(), 0, caller.clone())?;
		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: T::AccountId::encode(&crate::Pallet::<T>::get_pallet_account_id()).try_into().unwrap(),
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
		SContract::<T>::initialize_shard(
			RawOrigin::Signed(caller.clone()).into(), 0, encoded_calls.clone(),
			IPFS_CID_1.as_bytes().to_vec(),
			PUBLIC_KEY
		)?;
	}: shard_rollup ( RawOrigin::Signed(caller), 0, IPFS_CID_1.as_bytes().to_vec(), 10_000 ) verify { }
}

impl_benchmark_test_suite!(
	SContract,
	crate::mock::new_test_ext(),
	crate::mock::Test,
);
