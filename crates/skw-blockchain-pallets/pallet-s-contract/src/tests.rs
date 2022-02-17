use pallet_secrets::Event as SecretsEvent;
use crate::Event as SContractEvent;

use frame_support::{assert_ok};
use crate::mock::{Event, *};

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const ENCODED_CALL: &str = "1111111111222222222211111111112222222222";

type AccountId = u64;
const ALICE: AccountId = 1;

#[test]
fn it_register_secret_contracts() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			SContract::initialize_shard(
				Origin::root(), 0,
				ENCODED_CALL.as_bytes().to_vec(),
			)
		);

		assert_ok!(
			SContract::register_contract( 
				Origin::signed(ALICE), 
				IPFS_CID_1.as_bytes().to_vec(), 
				IPFS_CID_1.as_bytes().to_vec(),
				ENCODED_CALL.as_bytes().to_vec(),
				0,
			)
		);
		
		let events = System::events();
		assert! (events[0].event == Event::SContract(SContractEvent::ShardInitialized(0)));
		assert! (events[1].event == Event::Secrets(SecretsEvent::SecretContractRegistered(0)));

		let history = SContract::call_history_of(0, 1).unwrap();

		assert_eq! (history.len(), 2);
		assert_eq! (history[0], (ENCODED_CALL.as_bytes().to_vec(), AccountId::default()));
		assert_eq! (history[1], (ENCODED_CALL.as_bytes().to_vec(), ALICE));
	});
}
