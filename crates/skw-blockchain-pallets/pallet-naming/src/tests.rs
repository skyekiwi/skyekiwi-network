// use super::*;
// use frame_support::{assert_ok};
// use crate::mock::*;

// SBP M1 review: missing tests

// #[test]
// fn it_sets_empty_name() {
// 	new_test_ext().execute_with(|| {
// 		run_to_block(1);

// 		let name_hash = Blake2_128Concat::hash(b"test_name");
// 		Balance::set_balance(
// 			Origin::signed(1),
// 			Origin::signed(1),
// 			// 10000 units
// 			10_000_000_000_000_000
// 		);

// 		assert_ok!(
// 			// register the name for 10 * 5 = 50 blocks
// 			Naming::set_or_renew_name(
// 				Origin::signed(1), 
// 				name_hash,
// 				10
// 			)
// 		);

// 		assert_eq!(Balance::free_balance(1), 9_950_000_000_000_000);
// 		assert_eq!(Balance::reserved_balance(1), 50_000_000_000_000);
// 		assert_eq!(Naming::name_of(name_hash), Some(
// 			(1, 1 + 5 * 10, 50_000_000_000_000)
// 		));
// 	});
// }
