
//! Implementations for `nonfungibles` traits.

use super::*;
use frame_support::{
	traits::tokens::nonfungibles::{Inspect, InspectEnumerable, Mutate, Transfer},
	BoundedSlice,
};
use sp_runtime::DispatchResult;
use sp_std::convert::TryFrom;

impl<T: Config> Inspect<<T as frame_system::Config>::AccountId> for Pallet<T> {
	type InstanceId = T::TokenId;
	type ClassId = T::ClassId;

	fn owner(
		class: &Self::ClassId,
		instance: &Self::InstanceId,
	) -> Option<<T as frame_system::Config>::AccountId> {
        orml_nft::Pallet::<T>::tokens(class, instance).map(|a| a.owner)
		// <T as orml_nft::Config>::Tokens::<T>::get(class, instance).map(|a| a.owner)
	}

	fn class_owner(class: &Self::ClassId) -> Option<<T as frame_system::Config>::AccountId> {
		orml_nft::Pallet::<T>::classes(class).map(|a| a.owner)
	}

	/// Returns the attribute value of `instance` of `class` corresponding to `key`.
	///
	/// When `key` is empty, we return the instance metadata value.
	///
	/// By default this is `None`; no attributes are defined.
	fn attribute(
		class: &Self::ClassId,
		instance: &Self::InstanceId,
		key: &[u8],
	) -> Option<Vec<u8>> {
		if key.is_empty() {
			// We make the empty key map to the instance metadata value.
			orml_nft::Pallet::<T>::tokens(class, instance).map(|a| a.metadata.into())
		} else {
			return None
		}
	}

	/// Returns the attribute value of `instance` of `class` corresponding to `key`.
	///
	/// When `key` is empty, we return the instance metadata value.
	///
	/// By default this is `None`; no attributes are defined.
	fn class_attribute(class: &Self::ClassId, key: &[u8]) -> Option<Vec<u8>> {
		if key.is_empty() {
			// We make the empty key map to the instance metadata value.
            orml_nft::Pallet::<T>::classes(class).map(|a| a.metadata.into())
		} else {
            return None
		}
	}

	/// Returns `true` if the asset `instance` of `class` may be transferred.
	///
	/// Default implementation is that all assets are transferable.
	fn can_transfer(class: &Self::ClassId, instance: &Self::InstanceId) -> bool {
        match orml_nft::Pallet::<T>::classes(class) {
            Some(class) => class.data.properties.0.contains(ClassProperty::Transferable),
            _ => false,
        }
	}
}

// impl<T: Config<I>, I: 'static> Mutate<<T as SystemConfig>::AccountId> for Pallet<T, I> {
// 	fn mint_into(
// 		class: &Self::ClassId,
// 		instance: &Self::InstanceId,
// 		who: &T::AccountId,
// 	) -> DispatchResult {
// 		Self::do_mint(class.clone(), instance.clone(), who.clone(), |_| Ok(()))
// 	}

// 	fn burn_from(class: &Self::ClassId, instance: &Self::InstanceId) -> DispatchResult {
// 		Self::do_burn(class.clone(), instance.clone(), |_, _| Ok(()))
// 	}
// }

// impl<T: Config<I>, I: 'static> Transfer<T::AccountId> for Pallet<T, I> {
// 	fn transfer(
// 		class: &Self::ClassId,
// 		instance: &Self::InstanceId,
// 		destination: &T::AccountId,
// 	) -> DispatchResult {
// 		Self::do_transfer(class.clone(), instance.clone(), destination.clone(), |_, _| Ok(()))
// 	}
// }

// impl<T: Config<I>, I: 'static> InspectEnumerable<T::AccountId> for Pallet<T, I> {
// 	/// Returns an iterator of the asset classes in existence.
// 	///
// 	/// NOTE: iterating this list invokes a storage read per item.
// 	fn classes() -> Box<dyn Iterator<Item = Self::ClassId>> {
// 		Box::new(ClassMetadataOf::<T, I>::iter_keys())
// 	}

// 	/// Returns an iterator of the instances of an asset `class` in existence.
// 	///
// 	/// NOTE: iterating this list invokes a storage read per item.
// 	fn instances(class: &Self::ClassId) -> Box<dyn Iterator<Item = Self::InstanceId>> {
// 		Box::new(InstanceMetadataOf::<T, I>::iter_key_prefix(class))
// 	}

// 	/// Returns an iterator of the asset instances of all classes owned by `who`.
// 	///
// 	/// NOTE: iterating this list invokes a storage read per item.
// 	fn owned(who: &T::AccountId) -> Box<dyn Iterator<Item = (Self::ClassId, Self::InstanceId)>> {
// 		Box::new(Account::<T, I>::iter_key_prefix((who,)))
// 	}

// 	/// Returns an iterator of the asset instances of `class` owned by `who`.
// 	///
// 	/// NOTE: iterating this list invokes a storage read per item.
// 	fn owned_in_class(
// 		class: &Self::ClassId,
// 		who: &T::AccountId,
// 	) -> Box<dyn Iterator<Item = Self::InstanceId>> {
// 		Box::new(Account::<T, I>::iter_key_prefix((who, class)))
// 	}
// }
