use super::*;
use sp_core::H256;
use frame_support::{
	assert_ok, assert_noop, parameter_types, traits::Filter, Blake2_128Concat,
};
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use frame_system as system;
use crate as pallet_naming;
use pallet_balances as balances;


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
		Naming: pallet_naming::{Pallet, Call, Storage, Event<T>},
	}
);

fn run_to_block(n: u64) {
	while System::block_number() < n {
		Naming::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Naming::on_initialize(System::block_number());
	}
}
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
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
    pub const ReservationFee: u128 = 10_000_000_000_000;
		// pub const Day: BlockNumber = DAYS;
		pub const BlockPerPeriod: BlockNumber = 5;
		
		// reserve a slot no more than 5 years.
		pub const MaxPeriod: u32 = 1825;
}

impl pallet_naming::Config for Test {
    type ReservationFee = ReservationFee;
		type BlockPerPeriod = BlockPerPeriod;
		type MaxPeriod = MaxPeriod;

    type Event = Event;
    type Currency = Balances;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig {
		balances: Some(balances::GenesisConfig::<Test>{
			balances: vec![
				(1, 10_000_000_000_000_000), 
				(2, 10_000_000_000_000_000), 
				(3, 10_000_000_000_000_000), 
				(4, 10_000_000_000_000_000), 
				(5, 2_000_000_000_000_000)
			],
			vesting: vec![],
		}),
	}.build_storage().unwrap().into();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn it_sets_empty_name() {
	new_test_ext().execute_with(|| {
		run_to_block(1);

		let name_hash = Blake2_128Concat::hash(b"test_name");
		Balance::set_balance(
			Origin::signed(1),
			Origin::signed(1),
			// 10000 units
			10_000_000_000_000_000
		);

		assert_ok!(
			// register the name for 10 * 5 = 50 blocks
			Naming::set_or_renew_name(
				Origin::signed(1), 
				name_hash,
				10
			)
		);

		assert_eq!(Balance::free_balance(1), 9_950_000_000_000_000);
		assert_eq!(Balance::reserved_balance(1), 50_000_000_000_000);
		assert_eq!(Naming::name_of(name_hash), Some(
			(1, 1 + 5 * 10, 50_000_000_000_000)
		));
	});
}
