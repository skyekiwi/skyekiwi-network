use frame_support::{assert_ok};
use crate::mock::{*};

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];


#[test]
fn it_create_account() {

	let account1: AccountId = AccountId::from([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	let account2: AccountId = AccountId::from([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(
			SContract::add_authorized_shard_operator(
				Origin::root(), 0, account1.clone()
			)
		);

		assert_ok!(
			SContract::initialize_shard(
				Origin::signed(account1.clone()), 0,
				IPFS_CID_1.as_bytes().to_vec(),
				PUBLIC_KEY,
			)
		);

		assert_ok!(SAccount::create_account(Origin::signed(account2.clone()), 0));
	});
}

#[test]
fn it_force_create_account() {

	let account1: AccountId = AccountId::from([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	let account2: AccountId = AccountId::from([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(
			SContract::add_authorized_shard_operator(
				Origin::root(), 0, account1.clone()
			)
		);

		assert_ok!(
			SContract::initialize_shard(
				Origin::signed(account1.clone()), 0,
				IPFS_CID_1.as_bytes().to_vec(),
				PUBLIC_KEY,
			)
		);
		assert_ok!(SAccount::force_create_enclave_account(Origin::root(), 0, account2));
	});
}
