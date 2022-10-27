#![cfg(test)]
use pallet_secrets;
use crate as pallet_s_contract;

use frame_support::{
	traits::{Everything, ConstU32, ConstU64},
	PalletId,
	parameter_types
};
use frame_system::{EnsureRoot};
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
		Balances: pallet_balances::{Pallet, Call, Event<T>, Storage},
		Preimage: pallet_preimage::{Pallet, Call, Event<T>, Storage},
		Secrets: pallet_secrets::{Pallet, Call, Storage, Event<T>},
		SContract: pallet_s_contract::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
pub type AccountId = <<sp_runtime::MultiSignature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_preimage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<Self::AccountId>;
	type MaxSize = ConstU32<{ 4096 * 1024 }>; // PreimageMaxSize Taken from Polkadot as reference.
	type BaseDeposit = ConstU64<1>;
	type ByteDeposit = ConstU64<1>;
	type WeightInfo = ();
}

impl pallet_secrets::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Preimage = Preimage;
}

frame_support::parameter_types! {
	pub const SContractPalletId: PalletId = PalletId(*b"scontrac");
}
impl pallet_s_contract::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type MaxCallLength = ConstU32<100_1000>;
	type MinContractNameLength = ConstU32<1>;
	type MaxContractNameLength = ConstU32<32>;
	type MaxCallPerBlock = ConstU32<1_000>;
	type SContractRoot = SContractPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
