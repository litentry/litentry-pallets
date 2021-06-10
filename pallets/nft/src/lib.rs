#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use frame_support::{
	pallet_prelude::*,
	transactional,
};
use frame_system::pallet_prelude::*;
use orml_traits::NFT;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::StaticLookup,
	DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;

pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

pub type CID = Vec<u8>;

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq)]
pub enum ClassProperty {
	/// Token can be transferred
	Transferable = 0b00000001,
	/// Token can be burned
	Burnable = 0b00000010,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(
			<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?,
		))
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassData<BN, ID> {
	/// Property of token
	pub properties: Properties,
	/// from when user can claim this nft
	pub start_block: Option<BN>,
	/// till when user can claim this nft
	pub end_block: Option<BN>,
	/// merged from two class; if true, burn the two items 
	pub class_type: ClassType<ID>,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenData {
	/// if token is used to generate an advanced nft
	pub used: bool,
	/// 0 = common, otherwise say 1 = rare, 2 = super rare
	pub rarity: u8,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ClassType<ID> {
	Simple(u8),
	Claim(CID), // root
	Merge(ID, ID, bool),
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ orml_nft::Config<ClassData = ClassData<BlockNumberOf<Self>, ClassIdOf<Self>>, TokenData = TokenData>
	{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassIdNotFound,
		/// TokenId not found
		TokenIdNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Quantity is invalid. need >= 1
		InvalidQuantity,
		/// Property of class don't support transfer
		NonTransferable,
		/// Property of class don't support burn
		NonBurnable,
		/// Token not found
		TokenNotFound,
		/// Wrong class type
		WrongClassType,
		/// Merge nft's base nfts are not provided correctly
		WrongMergeBase,
		/// Use already used token to merge new token
		TokenUsed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Created NFT class. \[owner, class_id\]
		CreatedClass(T::AccountId, ClassIdOf<T>),
		/// Minted NFT token. \[from, to, class_id, quantity\]
		MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, u32),
		/// Transferred NFT token. \[from, to, class_id, token_id\]
		TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token. \[owner, class_id, token_id\]
		BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
		/// Burned NFT token with remark. \[owner, class_id, token_id, remark_hash\]
		BurnedTokenWithRemark(T::AccountId, ClassIdOf<T>, TokenIdOf<T>, T::Hash),
		/// Destroyed NFT class. \[owner, class_id\]
		DestroyedClass(T::AccountId, ClassIdOf<T>),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create NFT class, tokens belong to the class.
		///
		/// - `metadata`: external metadata
		/// - `properties`: class property, include `Transferable` `Burnable`
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		#[transactional]
		pub fn create_class(
			origin: OriginFor<T>, 
			metadata: CID, 
			properties: Properties, 
			start_block: Option<BlockNumberOf<T>>,
			end_block: Option<BlockNumberOf<T>>,
			class_type: ClassType<ClassIdOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let next_id = orml_nft::Pallet::<T>::next_class_id();

			// TODO charge

			match class_type {
				ClassType::Merge(id1, id2, burn) => {
					if !burn {
						ensure!(<orml_nft::Module<T>>::classes(id1).is_some(), Error::<T>::ClassIdNotFound);
						ensure!(<orml_nft::Module<T>>::classes(id2).is_some(), Error::<T>::ClassIdNotFound);
					} else {
						let class_info1 = orml_nft::Pallet::<T>::classes(id1).ok_or(Error::<T>::ClassIdNotFound)?;
						let class_info2 = orml_nft::Pallet::<T>::classes(id2).ok_or(Error::<T>::ClassIdNotFound)?;
	
						let data1 = class_info1.data;
						ensure!(
							data1.properties.0.contains(ClassProperty::Burnable),
							Error::<T>::NonBurnable
						);
						let data2 = class_info2.data;
						ensure!(
							data2.properties.0.contains(ClassProperty::Burnable),
							Error::<T>::NonBurnable
						);
					}
				}
				_ => {}
			}

			let data = ClassData {
				properties,
				start_block,
				end_block,
				class_type,
			};
			orml_nft::Pallet::<T>::create_class(&who, metadata, data)?;

			Self::deposit_event(Event::CreatedClass(who, next_id));
			Ok(().into())
		}

		/// Mint NFT token
		///
		/// - `to`: the token owner's account
		/// - `class_id`: token belong to the class id
		/// - `metadata`: external metadata
		/// - `quantity`: token quantity
		#[pallet::weight(<T as Config>::WeightInfo::mint(*quantity))]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source, // TODO why
			class_id: ClassIdOf<T>,
			metadata: CID,
			quantity: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			ensure!(quantity >= 1, Error::<T>::InvalidQuantity);
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			ensure!(who == class_info.owner, Error::<T>::NoPermission);

			match class_info.data.class_type {
				ClassType::Simple(_) => {}
				_ => {
					Err(Error::<T>::WrongClassType)?
				}
			}

			let data = TokenData {
				used: false,
				rarity: 0,
			};
			for _ in 0..quantity {
				orml_nft::Pallet::<T>::mint(&to, class_id, metadata.clone(), data.clone())?;
			}

			Self::deposit_event(Event::MintedToken(who, to, class_id, quantity));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::mint(1))]
		#[transactional]
		pub fn claim(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			proof: u32, // TODO: fix this
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;

			match class_info.data.class_type {
				ClassType::Claim(_) => {}
				_ => {
					Err(Error::<T>::WrongClassType)?
				}
			}

			// TODO: check if claimed
			// TODO: check proof

			// TODO: adjustible rarity
			let data = TokenData {
				used: false,
				rarity: 0,
			};

			// TODO: if metadata can change?
			let metadata = class_info.metadata;

			// TODO: add a claimed set

			orml_nft::Pallet::<T>::mint(&who, class_id, metadata, data)?;
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::mint(1))]
		#[transactional]
		pub fn merge(
			origin: OriginFor<T>,
			class_id: ClassIdOf<T>,
			token1: (ClassIdOf<T>, TokenIdOf<T>),
			token2: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;
			let merged_class_info = orml_nft::Pallet::<T>::classes(class_id).ok_or(Error::<T>::ClassIdNotFound)?;
			let mut burn = false;

			if let ClassType::Merge(id1, id2, b) = merged_class_info.data.class_type {
				ensure!(
					((id1 == token1.0) && (id2 == token2.0)) || ((id1 == token2.0) && (id2 == token1.0)),
					Error::<T>::WrongMergeBase,
				);
				burn = b;
			} else {
				Err(Error::<T>::WrongClassType)?
			}

			// get token 1 and 2
			let mut token_info1 = <orml_nft::Module<T>>::tokens(token1.0, token1.1).ok_or(Error::<T>::TokenNotFound)?;
			let mut token_info2 = <orml_nft::Module<T>>::tokens(token2.0, token2.1).ok_or(Error::<T>::TokenNotFound)?;

			// burn or set used of token 1 and 2
			if burn {
				Self::do_burn(&who, token1)?;
				Self::do_burn(&who, token2)?;
			} else {
				ensure!(!token_info1.data.used && !token_info2.data.used, Error::<T>::TokenUsed);
				token_info1.data.used = true;
				token_info2.data.used = true;
				orml_nft::Tokens::<T>::insert(token1.0, token1.1, token_info1);
				orml_nft::Tokens::<T>::insert(token2.0, token2.1, token_info2);
			}

			// mint new token
			// TODO: adjustible rarity
			let data = TokenData {
				used: false,
				rarity: 0,
			};

			// TODO: if metadata can change?
			let metadata = merged_class_info.metadata;

			orml_nft::Pallet::<T>::mint(&who, class_id, metadata, data)?;

			Ok(().into())
		}

		/// Transfer NFT token to another account
		///
		/// - `to`: the token owner's account
		/// - `token`: (class_id, token_id)
		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			token: (ClassIdOf<T>, TokenIdOf<T>),
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::do_transfer(&who, &to, token)?;
			Ok(().into())
		}

		/// Burn NFT token
		///
		/// - `token`: (class_id, token_id)
		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Self::do_burn(&who, token)?;
			Self::deposit_event(Event::BurnedToken(who, token.0, token.1));
			Ok(().into())
		}

	}
}

impl<T: Config> Pallet<T> {
	/// Ensured atomic.
	#[transactional]
	fn do_transfer(from: &T::AccountId, to: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Transferable),
			Error::<T>::NonTransferable
		);

		orml_nft::Pallet::<T>::transfer(from, to, token)?;

		Self::deposit_event(Event::TransferredToken(from.clone(), to.clone(), token.0, token.1));
		Ok(())
	}

	/// Ensured atomic.
	#[transactional]
	fn do_burn(who: &T::AccountId, token: (ClassIdOf<T>, TokenIdOf<T>)) -> DispatchResult {
		let class_info = orml_nft::Pallet::<T>::classes(token.0).ok_or(Error::<T>::ClassIdNotFound)?;
		let data = class_info.data;
		ensure!(
			data.properties.0.contains(ClassProperty::Burnable),
			Error::<T>::NonBurnable
		);

		let token_info = orml_nft::Pallet::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenIdNotFound)?;
		ensure!(*who == token_info.owner, Error::<T>::NoPermission);

		orml_nft::Pallet::<T>::burn(&who, token)?;

		Ok(())
	}
}

impl<T: Config> NFT<T::AccountId> for Pallet<T> {
	type ClassId = ClassIdOf<T>;
	type TokenId = TokenIdOf<T>;
	type Balance = u128;

	fn balance(who: &T::AccountId) -> Self::Balance {
		orml_nft::TokensByOwner::<T>::iter_prefix(who).count() as u128
	}

	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<T::AccountId> {
		orml_nft::Pallet::<T>::tokens(token.0, token.1).map(|t| t.owner)
	}

	fn transfer(from: &T::AccountId, to: &T::AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult {
		Self::do_transfer(from, to, token)
	}
}