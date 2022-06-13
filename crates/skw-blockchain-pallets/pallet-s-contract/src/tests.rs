use pallet_secrets::Event as SecretsEvent;
use crate::{Event as SContractEvent };
use frame_support::{assert_ok};
use crate::mock::{Event, *};

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";

#[test]
fn it_register_secret_contracts() {
	let account: AccountId = AccountId::from([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			SContract::add_authorized_shard_operator(
				Origin::root(), 0, account.clone()
			)
		);

		let mut all_calls = Vec::new();
		all_calls.push(skw_blockchain_primitives::types::Call {
			origin_public_key: SContract::get_pallet_account_id().into(),
			receipt_public_key: account.clone().into(),
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

		assert_ok!(
			SContract::initialize_shard(
				Origin::signed(account.clone()), 0,
				encoded_calls.clone(),
				IPFS_CID_1.as_bytes().to_vec(),
				SContract::get_pallet_account_id().into(),
			)
		);

		assert_ok!(
			SContract::register_contract( 
				Origin::signed(account.clone()),
				"contract_name".as_bytes().to_vec(),
				IPFS_CID_1.as_bytes().to_vec(), 
				encoded_calls.clone(),
				0,
			)
		);
		
		let events = System::events();
		assert! (events[0].event == Event::Secrets(SecretsEvent::SecretRegistered(0)));
		assert! (events[1].event == Event::SContract(SContractEvent::ShardInitialized(0)));
		assert! (events[2].event == Event::SContract(SContractEvent::SecretContractRegistered(
			0,
			"contract_name".as_bytes().to_vec(),
			1
		)));

		let history = SContract::call_history_of(0, 1).unwrap();
		assert_eq! (history.len(), 2);

		let init_call = SContract::call_record_of(history[0]).unwrap();
		let call_record = SContract::call_record_of(history[1]).unwrap();

		assert_eq! (init_call.0.into_inner(), encoded_calls.clone());
		assert_eq! (init_call.1, account.clone());
		assert_eq! (call_record.0.into_inner(), encoded_calls.clone());
		assert_eq! (call_record.1, account.clone());

		assert_ok!(
			SContract::force_push_call(
				Origin::root(),
				0,
				encoded_calls.clone(),
			) 
		);	
		
	});
}
