use pallet_secrets::Event as SecretsEvent;
use crate::{Event as SContractEvent, PublicKey};

use frame_support::{assert_ok};
use crate::mock::{Event, *};

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const ENCODED_CALL: &str = "1111111111222222222211111111112222222222";
const ENCODED_CALL2: &str = "22222222333333333333";
const PUBLIC_KEY: PublicKey = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
type AccountId = u64;
const ALICE: AccountId = 1;

#[test]
fn it_register_secret_contracts() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			SContract::add_authorized_shard_operator(
				Origin::root(), 0, ALICE
			)
		);

		assert_ok!(
			SContract::initialize_shard(
				Origin::signed(ALICE), 0,
				ENCODED_CALL.as_bytes().to_vec(),
				IPFS_CID_1.as_bytes().to_vec(),
				PUBLIC_KEY,
			)
		);

		assert_ok!(
			SContract::register_contract( 
				Origin::signed(ALICE),
				"contract_name".as_bytes().to_vec(),
				IPFS_CID_1.as_bytes().to_vec(), 
				ENCODED_CALL2.as_bytes().to_vec(),
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

		let init_call = SContract::call_record_of(0, history[0]).unwrap();
		let call_record = SContract::call_record_of(0, history[1]).unwrap();

		assert_eq! (init_call, (ENCODED_CALL.as_bytes().to_vec(), ALICE));
		assert_eq! (call_record, (ENCODED_CALL2.as_bytes().to_vec(), ALICE));

	});
}
