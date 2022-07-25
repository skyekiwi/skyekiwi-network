use pallet_secrets::Event as SecretsEvent;
use crate::{Event as SContractEvent };
use frame_support::{assert_ok};
use crate::mock::{Event, *};

const WASM_BLOB: &str = "123123123123123123123123";

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

		let  all_calls = Vec::new();
		let calls = skw_blockchain_primitives::types::Calls {
			ops: all_calls,
			block_number: Some(1),
			shard_id: 0
		};

		let encoded_calls = skw_blockchain_primitives::BorshSerialize::try_to_vec(&calls).unwrap();
		assert_ok!(
			SContract::initialize_shard(
				Origin::signed(account.clone()), 0,
				WASM_BLOB.as_bytes().to_vec(),
				SContract::get_pallet_account_id().into(),
			)
		);

		assert_ok!(
			SContract::register_contract( 
				Origin::signed(account.clone()),
				"contract_name".as_bytes().to_vec(),
				WASM_BLOB.as_bytes().to_vec(), 
				encoded_calls.clone(),
				0,
			)
		);
		
		let events = System::events();

		println!("{:?}", events);

		assert! (events[1].event == Event::Secrets(SecretsEvent::SecretRegistered(0)));
		assert! (events[2].event == Event::SContract(SContractEvent::ShardInitialized(0)));
		assert! (events[3].event == Event::SContract(SContractEvent::SecretContractRegistered(
			0,
			"contract_name".as_bytes().to_vec(),
			0,
		)));

		let history = SContract::call_history_of(0, 1).unwrap();
		assert_eq! (history.len(), 1);

		let call_record = SContract::call_record_of(history[0]).unwrap();
		assert_eq! (call_record.0.into_inner(), [1, 0, 0, 0, 109, 111, 100, 108, 115, 99, 111, 110, 116, 114, 97, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 100, 92, 81, 169, 219, 187, 161, 46, 234, 40, 69, 149, 163, 56, 230, 239, 204, 144, 154, 130, 132, 131, 114, 208, 52, 205, 224, 93, 13, 249, 42, 0, 4, 1, 10, 0, 0, 0, 1, 13, 0, 0, 0, 99, 111, 110, 116, 114, 97, 99, 116, 95, 110, 97, 109, 101, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0]);
		assert_eq! (call_record.1, account.clone());
	});
}
