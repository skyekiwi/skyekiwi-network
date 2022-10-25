use super::Event as ParentchainEvent;
use pallet_registry::Event as RegistryEvent;

use super::Error as ParentchainError;

use frame_support::{assert_ok, assert_noop};
use crate::mock::{RuntimeEvent, *};

const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

type AccountId = u64;
const ALICE: AccountId = 1;
const BOB: AccountId = 2;
// const CHARLIE: AccountId = 3;
// const DAVE: AccountId = 4;
// const FRED: AccountId = 5;

#[test]
fn it_submit_results() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = PUBLIC_KEY[..].to_vec();
		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(ALICE),  public_key.clone(), Vec::new() ) );
		
		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(ALICE), 0 ) );

		assert_ok!(
			Parentchain::set_shard_confirmation_threshold( 
				RuntimeOrigin::root(), 0,  1 //one confirmation
			)
		);

		assert_ok!(
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 1, 0,

				[0u8; 32],

				vec![], vec![]
			)
		);

		let events = System::events();
		assert!( 
			events[0].event == RuntimeEvent::Registry(RegistryEvent::SecretKeeperRegistered(1)) &&
			events[1].event == RuntimeEvent::Parentchain(ParentchainEvent::BlockSynced(1)) &&
			events[2].event == RuntimeEvent::Parentchain(ParentchainEvent::BlockConfirmed(1)) // threshold = 1, 1 sync = confirmed
			
		);
	});
}

// #[test]
// fn it_correctly_limit_beacon_turns_on_1_confirm() {

// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);

// 		let public_key = PUBLIC_KEY[..].to_vec();
// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(ALICE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(ALICE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(BOB),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(BOB), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(CHARLIE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(CHARLIE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(DAVE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(DAVE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(FRED),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(FRED), 0 ) );

// 		// 1 confirmation - only those who are in turn can submit
// 		assert_ok!(
// 			Parentchain::set_shard_confirmation_threshold( 
// 				RuntimeOrigin::root(), 0,  1 //one confirmation
// 			)
// 		);

// 		// For block_num 1 -> Alice can submit result
// 		assert_ok!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(ALICE), 1, 0,
// 				[0u8; 32], vec![], vec![]
// 			)
// 		);

// 		// Bob cannot submit for block_num 1
// 		assert_noop!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(BOB), 1, 0,
// 				[0u8; 32], vec![], vec![]
// 			),
// 			ParentchainError::<Test>::Unauthorized
// 		);

// 		// For block_num 2 -> Alice fails & Bob success
// 		assert_noop!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(ALICE), 2, 0,
// 				[0u8; 32], vec![], vec![]
// 			),
// 			ParentchainError::<Test>::Unauthorized
// 		);
// 		assert_ok!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(BOB), 2, 0,
// 				[0u8; 32], vec![], vec![]
// 			)
// 		);

// 		let events = System::events();
// 		assert!( 
// 			events[5].event == Event::Parentchain(ParentchainEvent::BlockSynced(1)) &&
// 			events[6].event == Event::Parentchain(ParentchainEvent::BlockConfirmed(1)) && // threshold = 1, 1 sync = confirmed
// 			events[7].event == Event::Parentchain(ParentchainEvent::BlockSynced(2)) &&
// 			events[8].event == Event::Parentchain(ParentchainEvent::BlockConfirmed(2)) // threshold = 1, 1 sync = confirmed
// 		);
// 	});
// }

// #[test]
// fn it_correctly_limit_beacon_turns_on_3_confirm() {

// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(1);

// 		let public_key = PUBLIC_KEY[..].to_vec();
// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(ALICE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(ALICE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(BOB),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(BOB), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(CHARLIE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(CHARLIE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(DAVE),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(DAVE), 0 ) );

// 		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(FRED),  public_key.clone(), Vec::new() ) );
// 		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(FRED), 0 ) );

// 		// now set threshold to 3
// 		assert_ok!(
// 			Parentchain::set_shard_confirmation_threshold( 
// 				RuntimeOrigin::root(), 0,  3 //three confirmation
// 			)
// 		);

// 		// for block_num 1; Alice, Bob, Charlie can submit; Dave will fail
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(ALICE), 1, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(BOB), 1, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(CHARLIE), 1, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_noop!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(DAVE), 1, 0,
// 				[0u8; 32], vec![], vec![]
// 			),
// 			ParentchainError::<Test>::Unauthorized
// 		);

// 		// for block_num 2; Alice fail; Bob, Charlie, Dave success
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(BOB), 2, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(CHARLIE), 2, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_ok!( Parentchain::submit_outcome( RuntimeOrigin::signed(DAVE), 2, 0, [0u8; 32], vec![], vec![] ) );
// 		assert_noop!(
// 			Parentchain::submit_outcome( 
// 				RuntimeOrigin::signed(ALICE), 2, 0,
// 				[0u8; 32], vec![], vec![]
// 			),
// 			ParentchainError::<Test>::Unauthorized
// 		);


// 		let events = System::events();
// 		assert!( 
// 			events[5].event == Event::Parentchain(ParentchainEvent::BlockSynced(1)) &&
// 			events[6].event == Event::Parentchain(ParentchainEvent::BlockConfirmed(1)) && // threshold = 1, 1 sync = confirmed
// 			events[7].event == Event::Parentchain(ParentchainEvent::BlockSynced(2)) &&
// 			events[8].event == Event::Parentchain(ParentchainEvent::BlockConfirmed(2)) // threshold = 1, 1 sync = confirmed
// 		);
// 	});
// }

#[test]
fn it_validates_outcome() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = PUBLIC_KEY[..].to_vec();
		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(ALICE),  public_key.clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(ALICE), 0 ) );

		assert_ok!(
			Parentchain::set_shard_confirmation_threshold( 
				RuntimeOrigin::root(), 0,  1 //one confirmation
			)
		);

		assert_ok!( 
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 
				1, 0, [0u8; 32],
				vec![], vec![] 
			) 
		);

		assert_ok!( 
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 
				2, 0, [0u8; 32],
				vec![ 11 ], vec![ [0u8; 100].to_vec() ] 
			) 
		);

		// len does not match
		assert_noop!(
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 3, 0,
				[0u8; 32],
				vec![ 11 ], vec![]
			),
			ParentchainError::<Test>::InvalidOutcome
		);

		// too many outcomes
		assert_noop!(
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 4, 0,
				[0u8; 32],
				vec![ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21], 
				vec![ 
					[0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), 
					[0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), 
					[0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), 
					[0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), [0u8; 100].to_vec(), 
					[0u8; 100].to_vec(),
				]
			),
			ParentchainError::<Test>::InvalidOutcome
		);

		// outcome too large
		assert_noop!(
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 
				5, 0, [0u8; 32],
				vec![ 22 ], vec![ [0u8; 1050].to_vec() ] 
			),
			ParentchainError::<Test>::InvalidOutcome
		);
	});
}

#[test]
fn it_validate_state_root_n_file_hash() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let public_key = PUBLIC_KEY[..].to_vec();
		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(ALICE),  public_key.clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(ALICE), 0 ) );

		assert_ok!( Registry::register_secret_keeper( RuntimeOrigin::signed(BOB),  public_key.clone(), Vec::new() ) );
		assert_ok!( Registry::register_running_shard( RuntimeOrigin::signed(BOB), 0 ) );

		assert_ok!(
			Parentchain::set_shard_confirmation_threshold( 
				RuntimeOrigin::root(), 0,  2 //two confirmation
			)
		);

		assert_ok!( 
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(ALICE), 
				1, 0, [0u8; 32],
				vec![], vec![] 
			) 
		);

		assert_noop!( 
			Parentchain::submit_outcome( 
				RuntimeOrigin::signed(BOB), 
				1, 0, [1u8; 32], 
				vec![], vec![] 
			),
			ParentchainError::<Test>::InconsistentState
		);
	});
}
