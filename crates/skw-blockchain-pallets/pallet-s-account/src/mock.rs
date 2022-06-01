#![cfg(test)]
use crate as pallet_s_account;
use pallet_s_contract;

use frame_support::{traits::{ConstU16, ConstU32, ConstU64, GenesisBuild}, PalletId};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup}, Permill,
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
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Treasury: pallet_treasury::{Pallet, Call, Storage, Event<T>},
		Secrets: pallet_secrets::{Pallet, Call, Storage, Event<T>},
		SAccount: pallet_s_account::{Pallet, Call, Storage, Event<T>},
		SContract: pallet_s_contract::{Pallet, Call, Storage, Event<T>},
	}
);

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

frame_support::parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const SContractPalletId: PalletId = PalletId(*b"scontrac");
}
impl pallet_treasury::Config for Test {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = frame_system::EnsureRoot<u64>;
	type RejectOrigin = frame_system::EnsureRoot<u64>;
	type Event = Event;
	type OnSlash = ();
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

impl frame_system::Config for Test {
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
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
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

impl pallet_s_contract::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type MaxCallLength = ConstU32<100_1000>;
	type MinContractNameLength = ConstU32<1>;
	type MaxContractNameLength = ConstU32<32>;
	type MaxCallPerBlock = ConstU32<1_000>;
	type SContractRoot = SContractPalletId;
}

impl pallet_s_account::Config for Test {
	type WeightInfo = ();
	type Event = Event;
	type ReservationRequirement = ConstU64<1>;
	type DefaultFaucet = ConstU32<1_000>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 20)],
	}.assimilate_storage(&mut t).unwrap();

	GenesisBuild::<Test>::assimilate_storage(&pallet_treasury::GenesisConfig, &mut t).unwrap();

	t.into()
}
