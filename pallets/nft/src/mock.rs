use super::*;
use crate as nft;
use frame_support::{
	parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::any::{Any, TypeId};

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
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>},
		Nft: nft::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Call = Call;
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const ClassCreationFee: u32 = CREATION_FEE;
	pub const Pot: AccountId32 = AccountId32::new([9u8; 32]);
}

impl nft::Config for Test {
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = ();
	type ClassCreationFee = ClassCreationFee;
	type Pot = Pot;
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Test {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = ClassData<BlockNumberOf<Self>, ClassIdOf<Self>>;
	type TokenData = TokenData;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

pub type NftError = nft::Error<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		<Nft as OnFinalize<u32>>::on_finalize(System::block_number());
		<System as OnFinalize<u32>>::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		<System as OnInitialize<u32>>::on_initialize(System::block_number());
		<Nft as OnInitialize<u32>>::on_initialize(System::block_number());
	}
}

pub fn events_filter<T: 'static>() -> Vec<Event> {
	let mut evt = System::events();

	evt.retain(|evt| if_right_events::<T>(&evt.event));
	return evt.into_iter().map(|evt| evt.event).collect::<Vec<_>>();
}

pub fn if_right_events<T: 'static>(evt: &Event) -> bool {
	if TypeId::of::<T>() == TypeId::of::<Event>() {
		return true;
	} else {
		match evt {
			Event::System(i) => return if_right_raw_events::<T>(i),
			Event::Balances(i) => return if_right_raw_events::<T>(i),
			Event::Nft(i) => return if_right_raw_events::<T>(i),
		}
	}
}

pub fn if_right_raw_events<T: 'static>(s: &dyn Any) -> bool {
	if let Some(_) = s.downcast_ref::<T>() {
		true
	} else {
		false
	}
}

pub fn get_vector<T>(vector: &Vec<T>, index: isize) -> &T {
	if index < 0 {
		return &vector[(vector.len() as isize + index) as usize];
	} else {
		return &vector[index as usize];
	}
}
