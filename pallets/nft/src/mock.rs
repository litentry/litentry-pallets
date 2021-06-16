use super::*;
use frame_support::{
	parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use crate as nft;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	generic,
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
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		OrmlNFT: orml_nft::{Module, Storage, Config<T>},
		Nft: nft::{Module, Call, Storage, Event<T>},
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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl nft::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

impl orml_nft::Config for Test {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = ClassData<BlockNumberOf<Self>, ClassIdOf<Self>>;
	type TokenData = TokenData;
}

pub type NftError = nft::Error<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn run_to_block(n: u32) {
    while System::block_number() < n {
        <Nft as OnFinalize::<u32>>::on_finalize(System::block_number());
        <System as OnFinalize::<u32>>::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        <System as OnInitialize::<u32>>::on_initialize(System::block_number());
		<Nft as OnInitialize::<u32>>::on_initialize(System::block_number());
    }
}

pub fn events() -> Vec<Event> {
	let evt = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();

	System::reset_events();

	evt
}