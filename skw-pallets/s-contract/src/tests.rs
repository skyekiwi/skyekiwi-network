use super::Event as SContractEvent;
use pallet_secrets::Event as SecretsEvent;

use frame_support::{assert_ok};
use crate::mock::{Event, *};
use sp_std::num::ParseIntError;

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const PUBLIC_KEY: &str = "38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f";
const ENCODED_CALL: &str = "1111111111222222222211111111112222222222";

type AccountId = u64;

const ALICE: AccountId = 1;

fn decode_hex_uncompressed(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(1)
		.map(|i| u8::from_str_radix(&s[i..i + 1], 16))
		.collect()
}
fn compress_hex_key(s: &Vec<u8>) -> Vec<u8> {
	(0..s.len())
		.step_by(2)
		.map(|i| s[i] * 16 + s[i + 1])
		.collect()
}

#[test]
fn it_register_secret_contracts() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = decode_hex_uncompressed(PUBLIC_KEY).unwrap();
		assert_ok!(
			SContract::register_contract( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec(), public_key.clone(), ENCODED_CALL.as_bytes().to_vec())
		);
		
		let pk = compress_hex_key(&public_key);
		assert! (System::events().iter().all(|evt| {
				evt.event == Event::Secrets(SecretsEvent::SecretContractRegistered(0, pk.clone()))
			})
		);

		let history = SContract::call_history_of(0).unwrap();

		assert_eq! (history.len(), 1);
		assert_eq! (history[0], (ENCODED_CALL.as_bytes().to_vec(), ALICE, false));
	});
}
