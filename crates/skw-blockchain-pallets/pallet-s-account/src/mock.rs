#![cfg(test)]
use crate as pallet_s_account;
use pallet_s_contract;

use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64},
};
use sp_runtime::{
	Perbill, Permill,
};

use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	PalletId,
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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
		Secrets: pallet_secrets::{Pallet, Call, Storage, Event<T>},
		SAccount: pallet_s_account::{Pallet, Call, Storage, Event<T>},
		SContract: pallet_s_contract::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
}
impl pallet_treasury::Config for Test {
	type PalletId = TreasuryPalletId;
	type Currency = Balance; 
	type ApproveOrigin = frame_system::EnsureRoot<u128>;
	type RejectOrigin = frame_system::EnsureRoot<u128>;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ConstU64<1>;
	type ProposalBondMaximum = ();
	type SpendPeriod = ConstU64<2>;
	type Burn = Burn;
	type BurnDestination = (); // Just gets burned.
	type WeightInfo = ();
	type SpendFunds = ();
	type MaxApprovals = ConstU32<100>;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

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
	pub const IPFSCIDLength: u32 = 46;
	pub const MaxActiveShards: u64 = 0;
}

impl pallet_secrets::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type IPFSCIDLength = IPFSCIDLength;
}

parameter_types! {
	pub const MaxCallLength: u32 = 512;
	pub const MinContractNameLength: u32 = 1;
	pub const MaxContractNameLength: u32 = 32;
}

impl pallet_s_contract::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type MaxCallLength = MaxCallLength;
	type MinContractNameLength = MinContractNameLength;
	type MaxContractNameLength = MaxContractNameLength;
}

parameter_types! {
	pub const ReservationRequirement: u32 = 1;
}

impl pallet_s_account::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type ReservationRequirement = ReservationRequirement;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 20)],
	}.assimilate_storage(&mut t).unwrap();

	GenesisBuild::<Test>::assimilate_storage(&pallet_treasury::GenesisConfig, &mut t).unwrap();

	t.into()
}
