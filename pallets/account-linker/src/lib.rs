//! # AccountLinker Pallet
//!
//! The AccountLinker pallet provides functionality for linking a Litentry account to account at
//! other networks. (currently support Ethereum (BSC), BTC and Polkadot ecosystem)
//!
//! ## Overview
//!
//! The AccountLinker pallet stores the linking relation between Litentry accounts and accounts at other
//! networks. It also offers extrinscs for user to update the linking relation. For each linking relation,
//! user may choose to freshly link new account or replace an existing linked account with a new provided one.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `link_eth` - Link an Ethereum address to a Litentry account providing a proof signature
//! from the private key of that Ethereum address.
//! * `link_btc` - Link an BTC address to a Litentry account providing a proof signature
//! from the private key of that BTC address.
//! * `link_polkadot` - Initiate a link request to link a Litentry address to another Litentry address
//! * `accept_polkadot` - Accept a pending `link_polkadot` request to link a Litentry address
//! to another Litentry address.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;
mod btc;
mod util_eth;
pub mod weights;

type EthAddress = [u8; 20];
// rsv signature
type Signature = [u8; 65];

#[frame_support::pallet]
pub mod pallet {
	use crate::*;
	use btc::{base58::ToBase58, witness::WitnessProgram};
	use codec::Encode;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
	use sp_std::prelude::*;
	use weights::WeightInfo;
	pub const EXPIRING_BLOCK_NUMBER_MAX: u32 = 10 * 60 * 24 * 30; // 30 days for 6s per block
	pub const MAX_ETH_LINKS: usize = 3;
	pub const MAX_BTC_LINKS: usize = 3;
	pub const MAX_POLKADOT_LINKS: usize = 3;

	enum BTCAddrType {
		Legacy,
		Segwit,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Ethereum address successfully linked. \[Lintentry account, Ethereum account\]
		EthAddressLinked(T::AccountId, Vec<u8>),
		/// BTC address successfully linked. \[Lintentry account, BTC account\]
		BtcAddressLinked(T::AccountId, Vec<u8>),
		/// Polkadot address successfully linked. \[Lintentry account, Polkadot account\]
		PolkadotAddressLinked(T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		// Cannot recover the signature
		EcdsaRecoverFailure,
		// Link request expired
		LinkRequestExpired,
		// Provided address mismatch the address recovered from signature recovery
		UnexpectedAddress,
		// Unexpected ethereum message length error
		UnexpectedEthMsgLength,
		// Invalid BTC address to link
		InvalidBTCAddress,
		// Expiration block number is too far away from now
		InvalidExpiringBlockNumber,
		// Try to resolve a wrong link_polkadot request
		WrongPendingRequest,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn eth_addresses)]
	pub(super) type EthereumLink<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<EthAddress>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn btc_addresses)]
	pub(super) type BitcoinLink<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn polkadot_addresses)]
	pub(super) type PolkadotLink<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn polkadot_pending)]
	pub(super) type PolkadotPending<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, u32), ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Link an Ethereum address to a Litentry account providing a proof signature from the private key
		/// of that Ethereum address.
		///
		/// The runtime needs to ensure that a malicious index can be handled correctly.
		/// Currently, when vec.len > MAX_ETH_LINKS, replacement will always happen at the final index.
		/// Otherwise it will use the next new slot unless index is valid against a currently available slot.
		///
		/// Parameters:
		/// - `account`: The Litentry address that is to be linked
		/// - `index`: The index of the linked Ethereum address that the user wants to replace with.
		/// - `addr_expected`: The intended Ethereum address to link to the origin's Litentry address
		/// - `expiring_block_number`: The block number after which this link request will expire
		/// - `sig`: The rsv-signature generated by the private key of the addr_expected
		///
		/// Emits `EthAddressLinked` event when successful.
		#[pallet::weight(T::WeightInfo::link_eth())]
		pub fn link_eth(
			origin: OriginFor<T>,
			account: T::AccountId,
			index: u32,
			addr_expected: EthAddress,
			expiring_block_number: T::BlockNumber,
			sig: Signature,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(expiring_block_number > current_block_number, Error::<T>::LinkRequestExpired);
			ensure!(
				(expiring_block_number - current_block_number) <
					T::BlockNumber::from(EXPIRING_BLOCK_NUMBER_MAX),
				Error::<T>::InvalidExpiringBlockNumber
			);

			let bytes = Self::generate_raw_message(&account, expiring_block_number);

			let hash =
				util_eth::eth_data_hash(bytes).map_err(|_| Error::<T>::UnexpectedEthMsgLength)?;

			let mut msg = [0u8; 32];
			msg[..32].copy_from_slice(&hash[..32]);

			let addr =
				util_eth::addr_from_sig(msg, sig).map_err(|_| Error::<T>::EcdsaRecoverFailure)?;
			ensure!(addr == addr_expected, Error::<T>::UnexpectedAddress);

			EthereumLink::<T>::mutate(&account, |addrs| {
				let index = index as usize;
				// NOTE: allow linking `MAX_ETH_LINKS` eth addresses.
				if (index >= addrs.len()) && (addrs.len() != MAX_ETH_LINKS) {
					addrs.push(addr.clone());
				} else if (index >= addrs.len()) && (addrs.len() == MAX_ETH_LINKS) {
					addrs[MAX_ETH_LINKS - 1] = addr.clone();
				} else {
					addrs[index] = addr.clone();
				}
			});

			Self::deposit_event(Event::EthAddressLinked(account, addr.to_vec()));

			Ok(().into())
		}

		/// Link a BTC address to a Litentry account providing a proof signature from the private key
		/// of that BTC address. The BTC address may either be a legacy P2PK one (started with b'1')
		/// or a Segwit P2PK one (started with b'bc').
		///
		/// The runtime needs to ensure that a malicious index can be handled correctly.
		/// Currently, when vec.len > MAX_ETH_LINKS, replacement will always happen at the final index.
		/// Otherwise it will use the next new slot unless index is valid against a currently available slot.
		///
		/// Parameters:
		/// - `account`: The Litentry address that is to be linked
		/// - `index`: The index of the linked BTC address that the user wants to replace with.
		/// - `addr_expected`: The intended BTC address to link to the origin's Litentry address
		/// - `expiring_block_number`: The block number after which this link request will expire
		/// - `sig`: The rsv-signature generated by the private key of the addr_expected
		///
		/// Emits `BtcAddressLinked` event when successful.
		#[pallet::weight(T::WeightInfo::link_btc())]
		pub fn link_btc(
			origin: OriginFor<T>,
			account: T::AccountId,
			index: u32,
			addr_expected: Vec<u8>,
			expiring_block_number: T::BlockNumber,
			sig: Signature,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(expiring_block_number > current_block_number, Error::<T>::LinkRequestExpired);
			ensure!(
				(expiring_block_number - current_block_number) <
					T::BlockNumber::from(EXPIRING_BLOCK_NUMBER_MAX),
				Error::<T>::InvalidExpiringBlockNumber
			);

			// TODO: we may enlarge this 2
			if addr_expected.len() < 2 {
				Err(Error::<T>::InvalidBTCAddress)?
			}

			let addr_type = if addr_expected[0] == b'1' {
				BTCAddrType::Legacy
			} else if addr_expected[0] == b'b' && addr_expected[1] == b'c' {
				// TODO: a better way?
				BTCAddrType::Segwit
			} else {
				Err(Error::<T>::InvalidBTCAddress)?
			};

			let bytes = Self::generate_raw_message(&account, expiring_block_number);

			// TODO: seems btc uses sha256???
			let hash = sp_io::hashing::keccak_256(&bytes);

			let mut msg = [0u8; 32];
			msg[..32].copy_from_slice(&hash[..32]);

			let pk = secp256k1_ecdsa_recover_compressed(&sig, &msg)
				.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;

			let addr = match addr_type {
				BTCAddrType::Legacy => btc::legacy::btc_addr_from_pk(&pk).to_base58(),
				// Native P2WPKH is a scriptPubKey of 22 bytes.
				// It starts with a OP_0, followed by a canonical push of the keyhash (i.e. 0x0014{20-byte keyhash})
				// keyhash is RIPEMD160(SHA256) of a compressed public key
				// https://bitcoincore.org/en/segwit_wallet_dev/
				BTCAddrType::Segwit => {
					let pk_hash = btc::legacy::hash160(&pk);
					let mut pk = [0u8; 22];
					pk[0] = 0;
					pk[1] = 20;
					pk[2..].copy_from_slice(&pk_hash);
					let wp = WitnessProgram::from_scriptpubkey(&pk.to_vec())
						.map_err(|_| Error::<T>::InvalidBTCAddress)?;
					wp.to_address(b"bc".to_vec()).map_err(|_| Error::<T>::InvalidBTCAddress)?
				},
			};

			ensure!(addr == addr_expected, Error::<T>::UnexpectedAddress);

			BitcoinLink::<T>::mutate(&account, |addrs| {
				let index = index as usize;
				// NOTE: allow linking `MAX_BTC_LINKS` btc addresses.
				if (index >= addrs.len()) && (addrs.len() != MAX_BTC_LINKS) {
					addrs.push(addr.clone());
				} else if (index >= addrs.len()) && (addrs.len() == MAX_BTC_LINKS) {
					addrs[MAX_BTC_LINKS - 1] = addr.clone();
				} else {
					addrs[index] = addr.clone();
				}
			});

			Self::deposit_event(Event::BtcAddressLinked(account, addr));

			Ok(().into())
		}

		/// Initiate a link request to link a Litentry address (= any account in Polkadot ecosystem)
		/// to another Litentry address (= any account in Polkadot ecosystem).
		///
		/// Parameters:
		/// - `account`: The Litentry address that is to be linked
		/// - `index`: The index of the linked Litentry address that the user wants to replace with.
		#[pallet::weight(T::WeightInfo::link_polkadot())]
		pub fn link_polkadot(
			origin: OriginFor<T>,
			account: T::AccountId,
			index: u32,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			// TODO: charge some fee

			<PolkadotPending<T>>::insert(origin, (account, index));

			Ok(().into())
		}

		/// Accept a pending `link_polkadot` request to link a Litentry address (= any account in Polkadot ecosystem)
		/// to another Litentry address (= any account in Polkadot ecosystem).
		///
		/// The runtime needs to ensure that a malicious index can be handled correctly.
		/// Currently, when vec.len > MAX_ETH_LINKS, replacement will always happen at the final index.
		/// Otherwise it will use the next new slot unless index is valid against a currently available slot.
		///
		/// Parameters:
		/// - `account`: The Litentry address that is to be linked
		///
		/// Emits `PolkadotAddressLinked` event when successful.
		#[pallet::weight(T::WeightInfo::accept_polkadot())]
		pub fn accept_polkadot(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			let (target, index) = Self::polkadot_pending(&account);
			ensure!(target == origin, Error::<T>::WrongPendingRequest);
			<PolkadotPending<T>>::remove(&origin);

			PolkadotLink::<T>::mutate(&account, |addrs| {
				let index = index as usize;
				// NOTE: allow linking `MAX_POLKADOT_LINKS` polkadot addresses.
				if (index >= addrs.len()) && (addrs.len() != MAX_POLKADOT_LINKS) {
					addrs.push(origin.clone());
				} else if (index >= addrs.len()) && (addrs.len() == MAX_POLKADOT_LINKS) {
					addrs[MAX_POLKADOT_LINKS - 1] = origin.clone();
				} else {
					addrs[index] = origin.clone();
				}
			});

			Self::deposit_event(Event::PolkadotAddressLinked(account, origin));

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Assemble the message that the user has signed
		/// Format: "Link Litentry: " + Litentry account + expiring block number
		fn generate_raw_message(
			account: &T::AccountId,
			expiring_block_number: T::BlockNumber,
		) -> Vec<u8> {
			let mut bytes = b"Link Litentry: ".encode();
			let mut account_vec = account.encode();
			let mut expiring_block_number_vec = expiring_block_number.encode();

			bytes.append(&mut account_vec);
			bytes.append(&mut expiring_block_number_vec);
			bytes
		}
	}
}
