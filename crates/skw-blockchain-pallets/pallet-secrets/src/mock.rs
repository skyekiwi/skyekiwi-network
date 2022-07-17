use crate as pallet_secrets;

use frame_support::traits::{ConstU32, ConstU64, Nothing};
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
	}
);
impl frame_system::Config for Test {
	type BaseCallFilter = Nothing;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_preimage::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<Self::AccountId>;
	type MaxSize = ConstU32<{ 4096 * 1024 }>; // PreimageMaxSize Taken from Polkadot as reference.
	type BaseDeposit = ConstU64<1>;
	type ByteDeposit = ConstU64<1>;
	type WeightInfo = ();
}

impl pallet_secrets::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type Preimage = Preimage;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
