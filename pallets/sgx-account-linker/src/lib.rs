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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

mod util_eth;
pub mod weights;

type EthAddress = [u8; 20];
// rsv signature
type Signature = [u8; 65];

#[frame_support::pallet]
pub mod pallet {
    use crate::*;
    use codec::Encode;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use weights::WeightInfo;
    pub const EXPIRING_BLOCK_NUMBER_MAX: u32 = 10 * 60 * 24 * 30; // 30 days for 6s per block
    pub const MAX_ETH_LINKS: usize = 3;
    pub const MAX_BTC_LINKS: usize = 3;
    pub const MAX_POLKADOT_LINKS: usize = 3;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
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
        // Can't get layer one block number
        LayerOneBlockNumberNotAvailable,
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
        /// of that Ethereum address. The extrinsic supposed to be executed in the sgx.
        ///
        /// The runtime needs to ensure that a malicious index can be handled correctly.
        /// Currently, when vec.len > MAX_ETH_LINKS, replacement will always happen at the final index.
        /// Otherwise it will use the next new slot unless index is valid against a currently available slot.
        ///
        /// Parameters:
        /// - `account`: The Litentry address that is to be linked
        /// - `index`: The index of the linked Ethereum address that the user wants to replace with.
        /// - `addr_expected`: The intended Ethereum address to link to the origin's Litentry address
        /// - `layer_one_block_number`: The current layer one block number
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
            layer_one_block_number: T::BlockNumber,
            expiring_block_number: T::BlockNumber,
            sig: Signature,
        ) -> DispatchResultWithPostInfo {
            // in sgx runtime, the account who want to link ethereum address don't have the balance to
            // submit extrinsic, the origin could be the root account
            let _ = ensure_signed(origin)?;
            Self::do_link_eth(
                account,
                index,
                addr_expected,
                expiring_block_number,
                layer_one_block_number,
                sig,
            )
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

        pub fn do_link_eth(
            account: T::AccountId,
            index: u32,
            addr_expected: EthAddress,
            expiring_block_number: T::BlockNumber,
            layer_one_blocknumber: T::BlockNumber,
            sig: Signature,
        ) -> DispatchResultWithPostInfo {
            log::error!("account: {:?}", account.encode());
            log::error!("addr_expected: {:?}", addr_expected);
            log::error!("exp number: {:?}", expiring_block_number);
            log::error!("sign: {:?}", sig);
            ensure!(
                expiring_block_number > layer_one_blocknumber,
                Error::<T>::LinkRequestExpired
            );
            ensure!(
                (expiring_block_number - layer_one_blocknumber)
                    < T::BlockNumber::from(EXPIRING_BLOCK_NUMBER_MAX),
                Error::<T>::InvalidExpiringBlockNumber
            );

            let bytes = Self::generate_raw_message(&account, expiring_block_number);

            let hash =
                util_eth::eth_data_hash(bytes).map_err(|_| Error::<T>::UnexpectedEthMsgLength)?;

            let mut msg = [0u8; 32];
            msg[..32].copy_from_slice(&hash[..32]);

            let addr: EthAddress =
                util_eth::addr_from_sig(msg, sig).map_err(|_| Error::<T>::EcdsaRecoverFailure)?;
            log::error!("Addr {:?}", addr);

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
    }
}
