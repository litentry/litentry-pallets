// Copyright 2020-2021 Litentry Technologies GmbH.
// This file is part of Litentry.
// 
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use scale_info::TypeInfo;
	use codec::{Decode, Encode};
	use frame_support::{
		fail,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, OnUnbalanced, StorageVersion, WithdrawReasons},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	pub use pallet_bridge as bridge;
	use sp_arithmetic::traits::SaturatedConversion;
	use sp_core::U256;
	use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
	use sp_std::prelude::*;

	type ResourceId = bridge::ResourceId;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct AssetInfo {
		pub dest_id: bridge::BridgeChainId,
		pub asset_identity: Vec<u8>,
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + bridge::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
		type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type NativeTokenResourceId: Get<ResourceId>;

		/// The handler to absorb the fee.
		type OnFeePay: OnUnbalanced<NegativeImbalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [chainId, min_fee, fee_scale]
		FeeUpdated(bridge::BridgeChainId, BalanceOf<T>, u32),
		/// [chainId, asset_identity, resource_id]
		AssetRegistered(bridge::BridgeChainId, Vec<u8>, bridge::ResourceId),
		/// [resource_id, amount]
		AssetMinted(bridge::ResourceId, BalanceOf<T>),
		/// [resource_id, amount]
		AssetBurned(bridge::ResourceId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		InvalidCommand,
		InvalidPayload,
		InvalidFeeOption,
		FeeOptionsMissing,
		InsufficientBalance,
		ResourceIdInUse,
		AssetNotRegistered,
		AccountNotExist,
		BalanceOverflow,
	}

	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> =
		StorageMap<_, Twox64Concat, bridge::BridgeChainId, (BalanceOf<T>, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn bridge_assets)]
	pub type BridgeAssets<T: Config> = StorageMap<_, Twox64Concat, bridge::ResourceId, AssetInfo>;

	#[pallet::storage]
	#[pallet::getter(fn bridge_balances)]
	pub type BridgeBalances<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		bridge::ResourceId,
		Twox64Concat,
		T::AccountId,
		BalanceOf<T>,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		

		/// Register an asset.
		#[pallet::weight(195_000_000)]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset_identity: Vec<u8>,
			dest_id: bridge::BridgeChainId,
		) -> DispatchResult {
			T::BridgeCommitteeOrigin::ensure_origin(origin)?;
			let resource_id = bridge::derive_resource_id(
				dest_id,
				&bridge::hashing::blake2_128(&asset_identity.to_vec()),
			);
			ensure!(
				!BridgeAssets::<T>::contains_key(resource_id),
				Error::<T>::ResourceIdInUse
			);
			BridgeAssets::<T>::insert(
				resource_id,
				AssetInfo {
					dest_id,
					asset_identity: asset_identity.clone(),
				},
			);
			Self::deposit_event(Event::AssetRegistered(dest_id, asset_identity, resource_id));
			Ok(())
		}

		//
		// Executable calls. These can be triggered by a bridge transfer initiated on another chain
		//

		/// Executes a simple currency transfer using the bridge account as the source
		#[pallet::weight(195_000_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
			rid: ResourceId,
		) -> DispatchResult {
			let source = T::BridgeOrigin::ensure_origin(origin)?;
			// transfer to bridge account from external accounts is not allowed.
			if source == to {
				fail!(Error::<T>::InvalidCommand);
			}

			if rid == T::NativeTokenResourceId::get() {
				// ERC20 PHA transfer
				<T as Config>::Currency::transfer(
					&source,
					&to,
					amount,
					ExistenceRequirement::AllowDeath,
				)?;
			} else {
				// Other ERC20 token transfer
				ensure!(
					Self::asset_balance(&rid, &source) >= amount,
					Error::<T>::InsufficientBalance
				);
				Self::do_asset_deposit(&rid, &to, amount).ok_or(Error::<T>::BalanceOverflow)?;
			}

			Ok(())
		}
	}

	// impl<T: Config> MessageOriginInfo for Pallet<T> {
	// 	type Config = T;
	// }

	impl<T: Config> Pallet<T> {
		pub fn asset_balance(asset: &bridge::ResourceId, who: &T::AccountId) -> BalanceOf<T> {
			BridgeBalances::<T>::get(asset, who).unwrap_or_default()
		}

		// TODO.wf: A more proper way to estimate fee
		pub fn calculate_fee(dest_id: bridge::BridgeChainId, amount: BalanceOf<T>) -> BalanceOf<T> {
			let (min_fee, fee_scale) = Self::bridge_fee(dest_id);
			let fee_estimated = amount * fee_scale.into() / 1000u32.into();
			if fee_estimated > min_fee {
				fee_estimated
			} else {
				min_fee
			}
		}

		/// Deposit specific amount assets into recipient account.
		///
		/// Assets would be withdrawn from bridge account and then deposit to
		/// recipient.
		/// Bridge account is treat as holding account of all assets.
		///
		/// DO NOT guarantee asset was registered
		/// DO NOT guarantee bridge account(e.g. hodling account) has enough balance
		pub fn do_asset_deposit(
			asset: &bridge::ResourceId,
			recipient: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Option<BalanceOf<T>> {
			let bridge_id = <bridge::Pallet<T>>::account_id();
			let holding_balance = BridgeBalances::<T>::get(asset, &bridge_id).unwrap_or_default();
			let recipient_balance = BridgeBalances::<T>::get(asset, recipient).unwrap_or_default();

			BridgeBalances::<T>::insert(asset, &bridge_id, holding_balance.checked_sub(&amount)?);
			BridgeBalances::<T>::insert(asset, recipient, recipient_balance.checked_add(&amount)?);

			Some(amount)
		}

		/// Withdraw specific amount assets from sender.
		///
		/// Assets would be withdrawn from the sender and then deposit to bridge account.
		/// Bridge account is treat as holding account of all assets.
		///
		/// DO NOT guarantee asset was registered
		/// DO NOT grarantee sender account has enough balance
		pub fn do_asset_withdraw(
			asset: &bridge::ResourceId,
			sender: &T::AccountId,
			amount: BalanceOf<T>,
		) -> Option<BalanceOf<T>> {
			let bridge_id = <bridge::Pallet<T>>::account_id();
			let holding_balance = BridgeBalances::<T>::get(asset, &bridge_id).unwrap_or_default();
			let recipient_balance = BridgeBalances::<T>::get(asset, sender).unwrap_or_default();

			BridgeBalances::<T>::insert(asset, sender, recipient_balance.checked_sub(&amount)?);
			BridgeBalances::<T>::insert(asset, &bridge_id, holding_balance.checked_add(&amount)?);

			Some(amount)
		}
	}
}
