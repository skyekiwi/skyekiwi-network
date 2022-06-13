use super::Event as RegistryEvent;

use frame_support::{assert_ok};
use crate::mock::{Event, *};

const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

type AccountId = u64;
const ALICE: AccountId = 1;
const BOB: AccountId = 2;
const CHARLIE: AccountId = 3;
const DAVE: AccountId = 4;
const FRED: AccountId = 5;

#[test]
fn it_register_secret_keeper() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				PUBLIC_KEY.to_vec().clone(),
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
		assert_eq! (Registry::is_valid_secret_keeper(&ALICE), true);
	});
}

#[test]
fn it_renews_registration() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				PUBLIC_KEY.to_vec().clone(),
				Vec::new()
			)
		);

		assert_ok!(
			Registry::renew_registration( 
				Origin::signed(ALICE), 
				PUBLIC_KEY.to_vec().clone(),
				Vec::new()
			)
		);

		assert_eq! (Registry::is_valid_secret_keeper(&ALICE), true);
	});
}


#[test]
fn it_removes_registration() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			Registry::register_secret_keeper( 
				Origin::signed(ALICE), 
				PUBLIC_KEY.to_vec().clone(),
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
		assert_eq! (Registry::is_valid_secret_keeper(&ALICE), false);
	});
}

#[test]
fn is_beacon_turn_test() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!( Registry::register_secret_keeper( Origin::signed(ALICE),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(ALICE), 0 ) );

		assert_ok!( Registry::register_secret_keeper( Origin::signed(BOB),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(BOB), 0 ) );

		assert_ok!( Registry::register_secret_keeper( Origin::signed(CHARLIE),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(CHARLIE), 0 ) );

		assert_ok!( Registry::register_secret_keeper( Origin::signed(DAVE),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(DAVE), 0 ) );

		assert_ok!( Registry::register_secret_keeper( Origin::signed(FRED),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(FRED), 0 ) );

		// threshold = 1; block_num = 1; Alice can submit, others cannot
		assert!( Registry::is_beacon_turn(1, &ALICE, 0, 1) == true);
		assert!( Registry::is_beacon_turn(1, &BOB, 0, 1) == false);
		assert!( Registry::is_beacon_turn(1, &CHARLIE, 0, 1) == false);

		// threshold = 1; block_num = 2; Bob can submit, others cannot
		assert!( Registry::is_beacon_turn(2, &ALICE, 0, 1) == false);
		assert!( Registry::is_beacon_turn(2, &BOB, 0, 1) == true);
		assert!( Registry::is_beacon_turn(2, &CHARLIE, 0, 1) == false);

		// threshold = 3; block_num = 1;
		// X X X _ _
		assert!( Registry::is_beacon_turn(1, &ALICE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(1, &BOB, 0, 3) == true);
		assert!( Registry::is_beacon_turn(1, &CHARLIE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(1, &DAVE, 0, 3) == false);
		assert!( Registry::is_beacon_turn(1, &FRED, 0, 3) == false);

		// threshold = 3; block_num = 2;
		// _ X X X _
		assert!( Registry::is_beacon_turn(2, &ALICE, 0, 3) == false);
		assert!( Registry::is_beacon_turn(2, &BOB, 0, 3) == true);
		assert!( Registry::is_beacon_turn(2, &CHARLIE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(2, &DAVE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(2, &FRED, 0, 3) == false);

		// threshold = 3; block_num = 4;
		// X _ _ X X 
		assert!( Registry::is_beacon_turn(4, &ALICE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(4, &BOB, 0, 3) == false);
		assert!( Registry::is_beacon_turn(4, &CHARLIE, 0, 3) == false);
		assert!( Registry::is_beacon_turn(4, &DAVE, 0, 3) == true);
		assert!( Registry::is_beacon_turn(4, &FRED, 0, 3) == true);
	});
}

#[test]
fn is_beacon_turn_test_signle_keeper() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!( Registry::register_secret_keeper( Origin::signed(ALICE),  PUBLIC_KEY.to_vec().clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( Origin::signed(ALICE), 0 ) );

		// threshold = 1; block_num = 1; Alice can submit, others cannot
		assert!( Registry::is_beacon_turn(1, &ALICE, 0, 1) == true);
		assert!( Registry::is_beacon_turn(2, &ALICE, 0, 1) == true);
		assert!( Registry::is_beacon_turn(3, &ALICE, 0, 1) == true);

	});
}

#[test]
fn insert_pk_for_user() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!( Registry::register_user_public_key( Origin::signed(ALICE),  PUBLIC_KEY.to_vec().clone()) );

		assert!( Registry::user_public_key_of(&ALICE).unwrap().to_vec() == PUBLIC_KEY.to_vec().clone());
	});
}
