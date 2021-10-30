use super::*;
use frame_support::{assert_noop, assert_ok};
use crate::mock::*;

const IPFS_CID_1: &'static str = "QmaibP61e3a4r6Bp895FQFB6ohqt5gMK4yeNy6yXxBmi8N";
const IPFS_CID_2: &'static str = "QmRTphmVWBbKAVNwuc8tjJjdxzJsxB7ovpGHyUUCE6Rnsb";

#[test]
fn it_register_secrets() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(
			Secrets::register_secret(
				Origin::signed(&ALICE),
				IPFS_CID_1.as_bytes().to_vec()
			)
		);

		assert_eq!(
			Secrets::owner(1),
			&ALICE
		);
	});
}
