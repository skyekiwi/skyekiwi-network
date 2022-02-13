#![cfg(test)]
use crate as pallet_parentchain;
use pallet_registry;

use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Registry: pallet_registry::{Pallet, Call, Storage, Event<T>},
		Parentchain: pallet_parentchain::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const RegistrationDuration: u32 = 1_000_000_000;
	pub const MaxActiveShards: u64 = 0;
}

impl pallet_registry::Config for Test {
	type Event = Event;
	type RegistrationDuration = RegistrationDuration;
	type MaxActiveShards = MaxActiveShards;
}

parameter_types! {
	pub const DeplayThreshold: u32 = 20;
	pub const MaxOutcomePerSubmission: u64 = 20;
	pub const MaxSizePerOutcome: u64 = 1024;
}

impl pallet_parentchain::Config for Test {
	type Event = Event;
	type DeplayThreshold = DeplayThreshold;
	type MaxOutcomePerSubmission = MaxOutcomePerSubmission;
	type MaxSizePerOutcome = MaxSizePerOutcome;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
