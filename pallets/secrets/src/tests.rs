use super::Event as SecretsEvent;
use frame_support::{assert_ok, assert_noop};
use crate::{mock::{Event, *}, Error};

const IPFS_CID_1: &'static str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const IPFS_CID_2: &'static str = "QmRTphmVWBbKAVNwuc8tjJjdxzJsxB7ovpGHyUUCE6Rnsb";

type AccountId = u64;

const ALICE: AccountId = 1;
const BOB: AccountId = 2;

#[test]
fn it_register_secrets() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(
			Secrets::register_secret( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec() )
		);

		assert! (System::events().iter().all(|evt| {
				evt.event == Event::Secrets(SecretsEvent::SecretRegistered(0))
			})
		);

		assert_eq! (Secrets::owner_of(0).unwrap(), ALICE);
		assert_eq! (Secrets::metadata_of(0).unwrap(), IPFS_CID_1.as_bytes().to_vec());
	});
}

#[test]
fn it_updates_metadata() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// 1. Alice register a secret w/ID = 0
		assert_ok!(
			Secrets::register_secret( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec() )
		);
		assert_eq! (Secrets::metadata_of(0).unwrap(), IPFS_CID_1.as_bytes().to_vec());

		// 2. Alice update the Metadata
		assert_ok!(
			Secrets::update_metadata( Origin::signed(ALICE), 0, IPFS_CID_2.as_bytes().to_vec() )
		);
		assert_eq! (Secrets::metadata_of(0).unwrap(), IPFS_CID_2.as_bytes().to_vec());
	});
}

#[test]
fn it_nominate_n_remove_member() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// 1. Alice register a secret w/ID = 0
		assert_ok!(
			Secrets::register_secret( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec() )
		);
		assert_eq! (Secrets::authorize_access(ALICE, 0), true);
		assert_eq! (Secrets::authorize_owner(ALICE, 0), true);

		// 2. Alice nominate Bob to be a member
		assert_ok!(
			Secrets::nominate_member( Origin::signed(ALICE), 0, BOB )
		);

		assert_eq! (Secrets::authorize_owner(ALICE, 0), true);
		assert_eq! (Secrets::authorize_access(ALICE, 0), true);
		assert_eq! (Secrets::authorize_owner(BOB, 0), false);
		assert_eq! (Secrets::authorize_access(BOB, 0), true);

		// 3. Bob cannot remove Alice
		assert_noop!(
			Secrets::remove_member( Origin::signed(BOB), 0, ALICE ),
			Error::<Test>::AccessDenied
		);

		// 3. Alice can remove Bob
		assert_ok!(
			Secrets::remove_member( Origin::signed(ALICE), 0, BOB )
		);
		assert_eq! (Secrets::authorize_access(BOB, 0), false);
	});
}

#[test]
fn members_can_update_metaedata() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// 1. Alice register a secret w/ID = 0
		assert_ok!(
			Secrets::register_secret( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec() )
		);

		// 2. Alice nominate Bob to be a member
		assert_ok!(
			Secrets::nominate_member( Origin::signed(ALICE), 0, BOB )
		);

		// 3. Bob can update metadata
		assert_ok!(
			Secrets::update_metadata( Origin::signed(BOB), 0, IPFS_CID_2.as_bytes().to_vec() )
		);
		assert_eq! (Secrets::metadata_of(0).unwrap(), IPFS_CID_2.as_bytes().to_vec());
	});
}

#[test]
fn owner_can_burn_secret() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		// 1. Alice register a secret w/ID = 0
		assert_ok!(
			Secrets::register_secret( Origin::signed(ALICE), IPFS_CID_1.as_bytes().to_vec() )
		);

		// 2. Alice nominate Bob to be a member
		assert_ok!(
			Secrets::nominate_member( Origin::signed(ALICE), 0, BOB )
		);

		// 3. Bob cannot burn secrets
		assert_noop!(
			Secrets::burn_secret( Origin::signed(BOB), 0 ),
			Error::<Test>::AccessDenied
		);

		// 4. Alice can burn secrets
		assert_ok!(
			Secrets::burn_secret( Origin::signed(ALICE), 0 )
		);
	});
}
