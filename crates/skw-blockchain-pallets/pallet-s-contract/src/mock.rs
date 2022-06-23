#![cfg(test)]
use pallet_secrets;
use crate as pallet_s_contract;

use frame_support::{
	traits::{ConstU16, ConstU32, ConstU64},
	PalletId,
};
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
		Secrets: pallet_secrets::{Pallet, Call, Storage, Event<T>},
		SContract: pallet_s_contract::{Pallet, Call, Storage, Event<T>},
	}
);
pub type AccountId = <<sp_runtime::MultiSignature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;

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
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_secrets::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type IPFSCIDLength = ConstU32<46>;
}


frame_support::parameter_types! {
	pub const SContractPalletId: PalletId = PalletId(*b"scontrac");
}
impl pallet_s_contract::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type MaxCallLength = ConstU32<100_1000>;
	type MinContractNameLength = ConstU32<1>;
	type MaxContractNameLength = ConstU32<32>;
	type MaxCallPerBlock = ConstU32<1_000>;
	type SContractRoot = SContractPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
