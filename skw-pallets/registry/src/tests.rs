use super::Event as RegistryEvent;

use frame_support::{assert_ok};
use crate::mock::{Event, *};
use sp_std::num::ParseIntError;

const PUBLIC_KEY: &str = "38d58afd1001bb265bce6ad24ff58239c62e1c98886cda9d7ccf41594f37d52f";

type AccountId = u64;
const ALICE: AccountId = 1;

fn decode_hex_uncompressed(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(1)
		.map(|i| u8::from_str_radix(&s[i..i + 1], 16))
		.collect()
}

#[test]
fn it_register_secret_keeper() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = decode_hex_uncompressed(PUBLIC_KEY).unwrap();
		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				public_key.clone(),
				Vec::new()
			)
		);
		
		assert! (System::events().iter().all(|evt| {
				evt.event == Event::Registry(RegistryEvent::SecretKeeperRegistered(ALICE))
			})
		);

		let all_secret_keepers = Registry::secret_keepers().unwrap();

		assert_eq! (all_secret_keepers.len(), 1);
		assert_eq! (all_secret_keepers[0], ALICE);
	});
}

#[test]
fn it_renews_registration() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = decode_hex_uncompressed(PUBLIC_KEY).unwrap();
		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				public_key.clone(),
				Vec::new()
			)
		);

		assert_ok!(
			Registry::renew_registration( 
				Origin::signed(ALICE), 
				public_key.clone(),
				Vec::new()
			)
		);
	});
}


#[test]
fn it_removes_registration() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = decode_hex_uncompressed(PUBLIC_KEY).unwrap();
		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				public_key.clone(),
				Vec::new()
			)
		);

		assert_ok!(
			Registry::remove_registration( 
				Origin::signed(ALICE),
			)
		);

		let all_secret_keepers = Registry::secret_keepers().unwrap();
		assert_eq! (all_secret_keepers.len(), 0);
	});
}
