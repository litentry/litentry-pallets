#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_offchain_worker.
pub trait WeightInfo {
	fn asset_claim() -> Weight;
}


/// Weights for pallet_offchain_worker using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn asset_claim() -> Weight {
		100 as Weight
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn asset_claim() -> Weight {
		100 as Weight
	}
}
