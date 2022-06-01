use super::Event as SAccountEvent;
use pallet_s_contract::Event as SContractEvent;

use super::Error as SAccountError;

use frame_support::{assert_ok, assert_noop};
use crate::mock::{Event, *};

const IPFS_CID_1: &str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const ENCODED_CALL: &str = "1111111111222222222211111111112222222222";
const ENCODED_CALL2: &str = "22222222333333333333";
const PUBLIC_KEY: [u8; 32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

type AccountId = u64;
const ALICE: AccountId = 1;
const BOB: AccountId = 2;

#[test]
fn it_create_account() {

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

		assert_ok!(SAccount::create_account(Origin::signed(BOB), 0));

		
	});
}

#[test]
fn it_force_create_account() {

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

		assert_ok!(SAccount::force_create_enclave_account(Origin::root(), 0, BOB));

		
	});
}