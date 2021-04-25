#![feature(prelude_import)]
//! # xrecovery Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The xrecovery pallet is an M-of-N social xrecovery tool for users to gain
//! access to their accounts if the private key or other authentication mechanism
//! is lost. Through this pallet, a user is able to make calls on-behalf-of another
//! account which they have recovered. The xrecovery process is protected by trusted
//! "friends" whom the original account owner chooses. A threshold (M) out of N
//! friends are needed to give another account access to the recoverable account.
//!
//! ### xrecovery Configuration
//!
//! The xrecovery process for each recoverable account can be configured by the account owner.
//! They are able to choose:
//! * `friends` - The list of friends that the account owner trusts to protect the
//!   xrecovery process for their account.
//! * `threshold` - The number of friends that need to approve a xrecovery process for
//!   the account to be successfully recovered.
//! * `delay_period` - The minimum number of blocks after the beginning of the xrecovery
//!   process that need to pass before the account can be successfully recovered.
//!
//! There is a configurable deposit that all users need to pay to create a xrecovery
//! configuration. This deposit is composed of a base deposit plus a multiplier for
//! the number of friends chosen. This deposit is returned in full when the account
//! owner removes their xrecovery configuration.
//!
//! ### xrecovery Life Cycle
//!
//! The intended life cycle of a successful xrecovery takes the following steps:
//! 1. The account owner calls `create_recovery` to set up a xrecovery configuration
//!    for their account.
//! 2. At some later time, the account owner loses access to their account and wants
//!    to recover it. Likely, they will need to create a new account and fund it with
//!    enough balance to support the transaction fees and the deposit for the
//!    xrecovery process.
//! 3. Using this new account, they call `initiate_recovery`.
//! 4. Then the account owner would contact their configured friends to vouch for
//!    the xrecovery attempt. The account owner would provide their old account id
//!    and the new account id, and friends would call `vouch_recovery` with those
//!    parameters.
//! 5. Once a threshold number of friends have vouched for the xrecovery attempt,
//!    the account owner needs to wait until the delay period has passed, starting
//!    when they initiated the xrecovery process.
//! 6. Now the account owner is able to call `claim_recovery`, which subsequently
//!    allows them to call `as_recovered` and directly make calls on-behalf-of the lost
//!    account.
//! 7. Using the now recovered account, the account owner can call `close_recovery`
//!    on the xrecovery process they opened, reclaiming the xrecovery deposit they
//!    placed.
//! 8. Then the account owner should then call `remove_recovery` to remove the xrecovery
//!    configuration on the recovered account and reclaim the xrecovery configuration
//!    deposit they placed.
//! 9. Using `as_recovered`, the account owner is able to call any other pallets
//!    to clean up their state and reclaim any reserved or locked funds. They
//!    can then transfer all funds from the recovered account to the new account.
//! 10. When the recovered account becomes reaped (i.e. its free and reserved
//!     balance drops to zero), the final xrecovery link is removed.
//!
//! ### Malicious xrecovery Attempts
//!
//! Initializing a the xrecovery process for a recoverable account is open and
//! permissionless. However, the xrecovery deposit is an economic deterrent that
//! should disincentivize would-be attackers from trying to maliciously recover
//! accounts.
//!
//! The xrecovery deposit can always be claimed by the account which is trying to
//! to be recovered. In the case of a malicious xrecovery attempt, the account
//! owner who still has access to their account can claim the deposit and
//! essentially punish the malicious user.
//!
//! Furthermore, the malicious xrecovery attempt can only be successful if the
//! attacker is also able to get enough friends to vouch for the xrecovery attempt.
//! In the case where the account owner prevents a malicious xrecovery process,
//! this pallet makes it near-zero cost to re-configure the xrecovery settings and
//! remove/replace friends who are acting inappropriately.
//!
//! ### Safety Considerations
//!
//! It is important to note that this is a powerful pallet that can compromise the
//! security of an account if used incorrectly. Some recommended practices for users
//! of this pallet are:
//!
//! * Configure a significant `delay_period` for your xrecovery process: As long as you
//!   have access to your recoverable account, you need only check the blockchain once
//!   every `delay_period` blocks to ensure that no xrecovery attempt is successful
//!   against your account. Using off-chain notification systems can help with this,
//!   but ultimately, setting a large `delay_period` means that even the most skilled
//!   attacker will need to wait this long before they can access your account.
//! * Use a high threshold of approvals: Setting a value of 1 for the threshold means
//!   that any of your friends would be able to recover your account. They would
//!   simply need to start a xrecovery process and approve their own process. Similarly,
//!   a threshold of 2 would mean that any 2 friends could work together to gain
//!   access to your account. The only way to prevent against these kinds of attacks
//!   is to choose a high threshold of approvals and select from a diverse friend
//!   group that would not be able to reasonably coordinate with one another.
//! * Reset your configuration over time: Since the entire deposit of creating a
//!   xrecovery configuration is returned to the user, the only cost of updating
//!   your xrecovery configuration is the transaction fees for the calls. Thus,
//!   it is strongly encouraged to regularly update your xrecovery configuration
//!   as your life changes and your relationship with new and existing friends
//!   change as well.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! #### For General Users
//!
//! * `create_recovery` - Create a xrecovery configuration for your account and make it recoverable.
//! * `initiate_recovery` - Start the xrecovery process for a recoverable account.
//!
//! #### For Friends of a Recoverable Account
//! * `vouch_recovery` - As a `friend` of a recoverable account, vouch for a xrecovery attempt on the account.
//!
//! #### For a User Who Successfully Recovered an Account
//!
//! * `claim_recovery` - Claim access to the account that you have successfully completed the xrecovery process for.
//! * `as_recovered` - Send a transaction as an account that you have recovered. See other functions below.
//!
//! #### For the Recoverable Account
//!
//! * `close_recovery` - Close an active xrecovery process for your account and reclaim the xrecovery deposit.
//! * `remove_recovery` - Remove the xrecovery configuration from the account, making it un-recoverable.
//!
//! #### For Super Users
//!
//! * `set_recovered` - The ROOT origin is able to skip the xrecovery process and directly allow
//!   one account to access another.
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
pub use pallet::*;
pub mod weights {
    #![allow(unused_parens)]
    #![allow(unused_imports)]
    use frame_support::{
        traits::Get,
        weights::{Weight, constants::RocksDbWeight},
    };
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
    impl WeightInfo for () {
        fn asset_claim() -> Weight {
            100 as Weight
        }
    }
}
use xcm::v0::{
    Junction::*,
    MultiAsset, MultiLocation, Order,
    Order::*,
    Xcm::{self, *},
};
pub mod pallet {
    use crate::*;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_runtime::{
        traits::{Dispatchable, SaturatedConversion, CheckedAdd, CheckedMul},
    };
    use codec::{Encode, Decode};
    use weights::WeightInfo;
    use frame_support::{
        pallet_prelude::*,
        Parameter, RuntimeDebug,
        weights::GetDispatchInfo,
        traits::{Currency, ReservableCurrency, Get, BalanceStatus},
        dispatch::DispatchResultWithPostInfo,
        dispatch::PostDispatchInfo,
    };
    use frame_system::{self as system, ensure_signed, ensure_root};
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    /// An active xrecovery process.
    pub struct ActiveRecovery<BlockNumber, Balance, AccountId> {
        /// The block number when the xrecovery process started.
        pub created: BlockNumber,
        /// The amount held in reserve of the `depositor`,
        /// To be returned once this xrecovery process is closed.
        pub deposit: Balance,
        /// The friends which have vouched so far. Always sorted.
        pub friends: Vec<AccountId>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::clone::Clone,
            Balance: ::core::clone::Clone,
            AccountId: ::core::clone::Clone,
        > ::core::clone::Clone for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn clone(&self) -> ActiveRecovery<BlockNumber, Balance, AccountId> {
            match *self {
                ActiveRecovery {
                    created: ref __self_0_0,
                    deposit: ref __self_0_1,
                    friends: ref __self_0_2,
                } => ActiveRecovery {
                    created: ::core::clone::Clone::clone(&(*__self_0_0)),
                    deposit: ::core::clone::Clone::clone(&(*__self_0_1)),
                    friends: ::core::clone::Clone::clone(&(*__self_0_2)),
                },
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> ::core::marker::StructuralEq
        for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<BlockNumber: ::core::cmp::Eq, Balance: ::core::cmp::Eq, AccountId: ::core::cmp::Eq>
        ::core::cmp::Eq for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<BlockNumber>;
                let _: ::core::cmp::AssertParamIsEq<Balance>;
                let _: ::core::cmp::AssertParamIsEq<Vec<AccountId>>;
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> ::core::marker::StructuralPartialEq
        for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::cmp::PartialEq,
            Balance: ::core::cmp::PartialEq,
            AccountId: ::core::cmp::PartialEq,
        > ::core::cmp::PartialEq for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn eq(&self, other: &ActiveRecovery<BlockNumber, Balance, AccountId>) -> bool {
            match *other {
                ActiveRecovery {
                    created: ref __self_1_0,
                    deposit: ref __self_1_1,
                    friends: ref __self_1_2,
                } => match *self {
                    ActiveRecovery {
                        created: ref __self_0_0,
                        deposit: ref __self_0_1,
                        friends: ref __self_0_2,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &ActiveRecovery<BlockNumber, Balance, AccountId>) -> bool {
            match *other {
                ActiveRecovery {
                    created: ref __self_1_0,
                    deposit: ref __self_1_1,
                    friends: ref __self_1_2,
                } => match *self {
                    ActiveRecovery {
                        created: ref __self_0_0,
                        deposit: ref __self_0_1,
                        friends: ref __self_0_2,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                    }
                },
            }
        }
    }
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::Encode
            for ActiveRecovery<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Encode,
            BlockNumber: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: _parity_scale_codec::Output + ?Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                _parity_scale_codec::Encode::encode_to(&self.created, __codec_dest_edqy);
                _parity_scale_codec::Encode::encode_to(&self.deposit, __codec_dest_edqy);
                _parity_scale_codec::Encode::encode_to(&self.friends, __codec_dest_edqy);
            }
        }
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::EncodeLike
            for ActiveRecovery<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Encode,
            BlockNumber: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
        {
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::Decode
            for ActiveRecovery<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Decode,
            BlockNumber: _parity_scale_codec::Decode,
            Balance: _parity_scale_codec::Decode,
            Balance: _parity_scale_codec::Decode,
            Vec<AccountId>: _parity_scale_codec::Decode,
            Vec<AccountId>: _parity_scale_codec::Decode,
        {
            fn decode<__CodecInputEdqy: _parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> core::result::Result<Self, _parity_scale_codec::Error> {
                Ok(ActiveRecovery::<BlockNumber, Balance, AccountId> {
                    created: {
                        let __codec_res_edqy = <BlockNumber as _parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `ActiveRecovery::created`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    deposit: {
                        let __codec_res_edqy =
                            <Balance as _parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `ActiveRecovery::deposit`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    friends: {
                        let __codec_res_edqy =
                            <Vec<AccountId> as _parity_scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `ActiveRecovery::friends`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                })
            }
        }
    };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::default::Default,
            Balance: ::core::default::Default,
            AccountId: ::core::default::Default,
        > ::core::default::Default for ActiveRecovery<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn default() -> ActiveRecovery<BlockNumber, Balance, AccountId> {
            ActiveRecovery {
                created: ::core::default::Default::default(),
                deposit: ::core::default::Default::default(),
                friends: ::core::default::Default::default(),
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> core::fmt::Debug
        for ActiveRecovery<BlockNumber, Balance, AccountId>
    where
        BlockNumber: core::fmt::Debug,
        Balance: core::fmt::Debug,
        AccountId: core::fmt::Debug,
    {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_struct("ActiveRecovery")
                .field("created", &self.created)
                .field("deposit", &self.deposit)
                .field("friends", &self.friends)
                .finish()
        }
    }
    /// Configuration for recovering an account.
    pub struct RecoveryConfig<BlockNumber, Balance, AccountId> {
        /// The minimum number of blocks since the start of the xrecovery process before the account
        /// can be recovered.
        pub delay_period: BlockNumber,
        /// The amount held in reserve of the `depositor`,
        /// to be returned once this configuration is removed.
        pub deposit: Balance,
        /// The list of friends which can help recover an account. Always sorted.
        pub friends: Vec<AccountId>,
        /// The number of approving friends needed to recover an account.
        pub threshold: u16,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::clone::Clone,
            Balance: ::core::clone::Clone,
            AccountId: ::core::clone::Clone,
        > ::core::clone::Clone for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn clone(&self) -> RecoveryConfig<BlockNumber, Balance, AccountId> {
            match *self {
                RecoveryConfig {
                    delay_period: ref __self_0_0,
                    deposit: ref __self_0_1,
                    friends: ref __self_0_2,
                    threshold: ref __self_0_3,
                } => RecoveryConfig {
                    delay_period: ::core::clone::Clone::clone(&(*__self_0_0)),
                    deposit: ::core::clone::Clone::clone(&(*__self_0_1)),
                    friends: ::core::clone::Clone::clone(&(*__self_0_2)),
                    threshold: ::core::clone::Clone::clone(&(*__self_0_3)),
                },
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> ::core::marker::StructuralEq
        for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<BlockNumber: ::core::cmp::Eq, Balance: ::core::cmp::Eq, AccountId: ::core::cmp::Eq>
        ::core::cmp::Eq for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<BlockNumber>;
                let _: ::core::cmp::AssertParamIsEq<Balance>;
                let _: ::core::cmp::AssertParamIsEq<Vec<AccountId>>;
                let _: ::core::cmp::AssertParamIsEq<u16>;
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> ::core::marker::StructuralPartialEq
        for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::cmp::PartialEq,
            Balance: ::core::cmp::PartialEq,
            AccountId: ::core::cmp::PartialEq,
        > ::core::cmp::PartialEq for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn eq(&self, other: &RecoveryConfig<BlockNumber, Balance, AccountId>) -> bool {
            match *other {
                RecoveryConfig {
                    delay_period: ref __self_1_0,
                    deposit: ref __self_1_1,
                    friends: ref __self_1_2,
                    threshold: ref __self_1_3,
                } => match *self {
                    RecoveryConfig {
                        delay_period: ref __self_0_0,
                        deposit: ref __self_0_1,
                        friends: ref __self_0_2,
                        threshold: ref __self_0_3,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                            && (*__self_0_3) == (*__self_1_3)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &RecoveryConfig<BlockNumber, Balance, AccountId>) -> bool {
            match *other {
                RecoveryConfig {
                    delay_period: ref __self_1_0,
                    deposit: ref __self_1_1,
                    friends: ref __self_1_2,
                    threshold: ref __self_1_3,
                } => match *self {
                    RecoveryConfig {
                        delay_period: ref __self_0_0,
                        deposit: ref __self_0_1,
                        friends: ref __self_0_2,
                        threshold: ref __self_0_3,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                            || (*__self_0_3) != (*__self_1_3)
                    }
                },
            }
        }
    }
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::Encode
            for RecoveryConfig<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Encode,
            BlockNumber: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: _parity_scale_codec::Output + ?Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                _parity_scale_codec::Encode::encode_to(&self.delay_period, __codec_dest_edqy);
                _parity_scale_codec::Encode::encode_to(&self.deposit, __codec_dest_edqy);
                _parity_scale_codec::Encode::encode_to(&self.friends, __codec_dest_edqy);
                _parity_scale_codec::Encode::encode_to(&self.threshold, __codec_dest_edqy);
            }
        }
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::EncodeLike
            for RecoveryConfig<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Encode,
            BlockNumber: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Balance: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
            Vec<AccountId>: _parity_scale_codec::Encode,
        {
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<BlockNumber, Balance, AccountId> _parity_scale_codec::Decode
            for RecoveryConfig<BlockNumber, Balance, AccountId>
        where
            BlockNumber: _parity_scale_codec::Decode,
            BlockNumber: _parity_scale_codec::Decode,
            Balance: _parity_scale_codec::Decode,
            Balance: _parity_scale_codec::Decode,
            Vec<AccountId>: _parity_scale_codec::Decode,
            Vec<AccountId>: _parity_scale_codec::Decode,
        {
            fn decode<__CodecInputEdqy: _parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> core::result::Result<Self, _parity_scale_codec::Error> {
                Ok(RecoveryConfig::<BlockNumber, Balance, AccountId> {
                    delay_period: {
                        let __codec_res_edqy = <BlockNumber as _parity_scale_codec::Decode>::decode(
                            __codec_input_edqy,
                        );
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(
                                    e.chain("Could not decode `RecoveryConfig::delay_period`")
                                )
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    deposit: {
                        let __codec_res_edqy =
                            <Balance as _parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `RecoveryConfig::deposit`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    friends: {
                        let __codec_res_edqy =
                            <Vec<AccountId> as _parity_scale_codec::Decode>::decode(
                                __codec_input_edqy,
                            );
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `RecoveryConfig::friends`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                    threshold: {
                        let __codec_res_edqy =
                            <u16 as _parity_scale_codec::Decode>::decode(__codec_input_edqy);
                        match __codec_res_edqy {
                            Err(e) => {
                                return Err(e.chain("Could not decode `RecoveryConfig::threshold`"))
                            }
                            Ok(__codec_res_edqy) => __codec_res_edqy,
                        }
                    },
                })
            }
        }
    };
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<
            BlockNumber: ::core::default::Default,
            Balance: ::core::default::Default,
            AccountId: ::core::default::Default,
        > ::core::default::Default for RecoveryConfig<BlockNumber, Balance, AccountId>
    {
        #[inline]
        fn default() -> RecoveryConfig<BlockNumber, Balance, AccountId> {
            RecoveryConfig {
                delay_period: ::core::default::Default::default(),
                deposit: ::core::default::Default::default(),
                friends: ::core::default::Default::default(),
                threshold: ::core::default::Default::default(),
            }
        }
    }
    impl<BlockNumber, Balance, AccountId> core::fmt::Debug
        for RecoveryConfig<BlockNumber, Balance, AccountId>
    where
        BlockNumber: core::fmt::Debug,
        Balance: core::fmt::Debug,
        AccountId: core::fmt::Debug,
    {
        fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
            fmt.debug_struct("RecoveryConfig")
                .field("delay_period", &self.delay_period)
                .field("deposit", &self.deposit)
                .field("friends", &self.friends)
                .field("threshold", &self.threshold)
                .finish()
        }
    }
    /// Configuration trait.
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Weights definition.
        type WeightInfo: WeightInfo;
        /// The overarching call type.
        type Call: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + GetDispatchInfo;
        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;
        /// The base amount of currency needed to reserve for creating a xrecovery configuration.
        ///
        /// This is held for an additional storage item whose value size is
        /// `2 + sizeof(BlockNumber, Balance)` bytes.
        type ConfigDepositBase: Get<BalanceOf<Self>>;
        /// The amount of currency needed per additional user when creating a xrecovery configuration.
        ///
        /// This is held for adding `sizeof(AccountId)` bytes more into a pre-existing storage value.
        type FriendDepositFactor: Get<BalanceOf<Self>>;
        /// The maximum amount of friends allowed in a xrecovery configuration.
        type MaxFriends: Get<u16>;
        /// The base amount of currency needed to reserve for starting a xrecovery.
        ///
        /// This is primarily held for deterring malicious xrecovery attempts, and should
        /// have a value large enough that a bad actor would choose not to place this
        /// deposit. It also acts to fund additional storage item whose value size is
        /// `sizeof(BlockNumber, Balance + T * AccountId)` bytes. Where T is a configurable
        /// threshold.
        type RecoveryDeposit: Get<BalanceOf<Self>>;
    }
    pub enum Event<T: Config> {
        /// A xrecovery process has been set up for an \[account\].
        RecoveryCreated(T::AccountId),
        /// A xrecovery process has been initiated for lost account by rescuer account.
        /// \[lost, rescuer\]
        RecoveryInitiated(T::AccountId, T::AccountId),
        /// A xrecovery process for lost account by rescuer account has been vouched for by sender.
        /// \[lost, rescuer, sender\]
        RecoveryVouched(T::AccountId, T::AccountId, T::AccountId),
        /// A xrecovery process for lost account by rescuer account has been closed.
        /// \[lost, rescuer\]
        RecoveryClosed(T::AccountId, T::AccountId),
        /// Lost account has been successfully recovered by rescuer account.
        /// \[lost, rescuer\]
        AccountRecovered(T::AccountId, T::AccountId),
        /// A xrecovery process has been removed for an \[account\].
        RecoveryRemoved(T::AccountId),
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T)>,
            frame_support::Never,
        ),
    }
    const _: () = {
        impl<T: Config> core::clone::Clone for Event<T> {
            fn clone(&self) -> Self {
                match self {
                    Self::RecoveryCreated(ref _0) => {
                        Self::RecoveryCreated(core::clone::Clone::clone(_0))
                    }
                    Self::RecoveryInitiated(ref _0, ref _1) => Self::RecoveryInitiated(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::RecoveryVouched(ref _0, ref _1, ref _2) => Self::RecoveryVouched(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                        core::clone::Clone::clone(_2),
                    ),
                    Self::RecoveryClosed(ref _0, ref _1) => Self::RecoveryClosed(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::AccountRecovered(ref _0, ref _1) => Self::AccountRecovered(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::RecoveryRemoved(ref _0) => {
                        Self::RecoveryRemoved(core::clone::Clone::clone(_0))
                    }
                    Self::__Ignore(ref _0, ref _1) => {
                        Self::__Ignore(core::clone::Clone::clone(_0), core::clone::Clone::clone(_1))
                    }
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::cmp::Eq for Event<T> {}
    };
    const _: () = {
        impl<T: Config> core::cmp::PartialEq for Event<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::RecoveryCreated(_0), Self::RecoveryCreated(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (
                        Self::RecoveryInitiated(_0, _1),
                        Self::RecoveryInitiated(_0_other, _1_other),
                    ) => true && _0 == _0_other && _1 == _1_other,
                    (
                        Self::RecoveryVouched(_0, _1, _2),
                        Self::RecoveryVouched(_0_other, _1_other, _2_other),
                    ) => true && _0 == _0_other && _1 == _1_other && _2 == _2_other,
                    (Self::RecoveryClosed(_0, _1), Self::RecoveryClosed(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (
                        Self::AccountRecovered(_0, _1),
                        Self::AccountRecovered(_0_other, _1_other),
                    ) => true && _0 == _0_other && _1 == _1_other,
                    (Self::RecoveryRemoved(_0), Self::RecoveryRemoved(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (Self::__Ignore(_0, _1), Self::__Ignore(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::RecoveryCreated { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::RecoveryCreated { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::RecoveryCreated { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::RecoveryCreated { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::RecoveryCreated { .. }, Self::RecoveryRemoved { .. }) => false,
                    (Self::RecoveryCreated { .. }, Self::__Ignore { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::RecoveryRemoved { .. }) => false,
                    (Self::RecoveryInitiated { .. }, Self::__Ignore { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::RecoveryRemoved { .. }) => false,
                    (Self::RecoveryVouched { .. }, Self::__Ignore { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::RecoveryRemoved { .. }) => false,
                    (Self::RecoveryClosed { .. }, Self::__Ignore { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::RecoveryRemoved { .. }) => false,
                    (Self::AccountRecovered { .. }, Self::__Ignore { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::RecoveryRemoved { .. }, Self::__Ignore { .. }) => false,
                    (Self::__Ignore { .. }, Self::RecoveryCreated { .. }) => false,
                    (Self::__Ignore { .. }, Self::RecoveryInitiated { .. }) => false,
                    (Self::__Ignore { .. }, Self::RecoveryVouched { .. }) => false,
                    (Self::__Ignore { .. }, Self::RecoveryClosed { .. }) => false,
                    (Self::__Ignore { .. }, Self::AccountRecovered { .. }) => false,
                    (Self::__Ignore { .. }, Self::RecoveryRemoved { .. }) => false,
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::fmt::Debug for Event<T> {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                match *self {
                    Self::RecoveryCreated(ref _0) => fmt
                        .debug_tuple("Event::RecoveryCreated")
                        .field(&_0)
                        .finish(),
                    Self::RecoveryInitiated(ref _0, ref _1) => fmt
                        .debug_tuple("Event::RecoveryInitiated")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::RecoveryVouched(ref _0, ref _1, ref _2) => fmt
                        .debug_tuple("Event::RecoveryVouched")
                        .field(&_0)
                        .field(&_1)
                        .field(&_2)
                        .finish(),
                    Self::RecoveryClosed(ref _0, ref _1) => fmt
                        .debug_tuple("Event::RecoveryClosed")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::AccountRecovered(ref _0, ref _1) => fmt
                        .debug_tuple("Event::AccountRecovered")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::RecoveryRemoved(ref _0) => fmt
                        .debug_tuple("Event::RecoveryRemoved")
                        .field(&_0)
                        .finish(),
                    Self::__Ignore(ref _0, ref _1) => fmt
                        .debug_tuple("Event::__Ignore")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                }
            }
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<T: Config> _parity_scale_codec::Encode for Event<T>
        where
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: _parity_scale_codec::Output + ?Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Event::RecoveryCreated(ref aa) => {
                        __codec_dest_edqy.push_byte(0usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    Event::RecoveryInitiated(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(1usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Event::RecoveryVouched(ref aa, ref ba, ref ca) => {
                        __codec_dest_edqy.push_byte(2usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ca, __codec_dest_edqy);
                    }
                    Event::RecoveryClosed(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(3usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Event::AccountRecovered(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(4usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Event::RecoveryRemoved(ref aa) => {
                        __codec_dest_edqy.push_byte(5usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        impl<T: Config> _parity_scale_codec::EncodeLike for Event<T>
        where
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
        {
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<T: Config> _parity_scale_codec::Decode for Event<T>
        where
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
        {
            fn decode<__CodecInputEdqy: _parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> core::result::Result<Self, _parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| e.chain("Could not decode `Event`, failed to read variant byte"))?
                {
                    __codec_x_edqy if __codec_x_edqy == 0usize as u8 => {
                        Ok(Event::<T>::RecoveryCreated({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(
                                        e.chain("Could not decode `Event::RecoveryCreated.0`")
                                    )
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as u8 => {
                        Ok(Event::<T>::RecoveryInitiated(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(e.chain(
                                            "Could not decode `Event::RecoveryInitiated.0`",
                                        ))
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(e.chain(
                                            "Could not decode `Event::RecoveryInitiated.1`",
                                        ))
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 2usize as u8 => {
                        Ok(Event::<T>::RecoveryVouched(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::RecoveryVouched.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::RecoveryVouched.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::RecoveryVouched.2`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 3usize as u8 => {
                        Ok(Event::<T>::RecoveryClosed(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::RecoveryClosed.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::RecoveryClosed.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 4usize as u8 => {
                        Ok(Event::<T>::AccountRecovered(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::AccountRecovered.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Event::AccountRecovered.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 5usize as u8 => {
                        Ok(Event::<T>::RecoveryRemoved({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(
                                        e.chain("Could not decode `Event::RecoveryRemoved.0`")
                                    )
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    _ => Err("Could not decode `Event`, variant doesn\'t exist".into()),
                }
            }
        }
    };
    pub enum Error<T> {
        #[doc(hidden)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T)>,
            frame_support::Never,
        ),
        /// User is not allowed to make a call on behalf of this account
        NotAllowed,
        /// Threshold must be greater than zero
        ZeroThreshold,
        /// Friends list must be greater than zero and threshold
        NotEnoughFriends,
        /// Friends list must be less than max friends
        MaxFriends,
        /// Friends list must be sorted and free of duplicates
        NotSorted,
        /// This account is not set up for xrecovery
        NotRecoverable,
        /// This account is already set up for xrecovery
        AlreadyRecoverable,
        /// A xrecovery process has already started for this account
        AlreadyStarted,
        /// A xrecovery process has not started for this rescuer
        NotStarted,
        /// This account is not a friend who can vouch
        NotFriend,
        /// The friend must wait until the delay period to vouch for this xrecovery
        DelayPeriod,
        /// This user has already vouched for this xrecovery
        AlreadyVouched,
        /// The threshold for recovering this account has not been met
        Threshold,
        /// There are still active xrecovery attempts that need to be closed
        StillActive,
        /// There was an overflow in a calculation
        Overflow,
        /// This account is already set up for xrecovery
        AlreadyProxy,
        /// Some internal state is broken.
        BadState,
    }
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    pub struct Pallet<T>(frame_support::sp_std::marker::PhantomData<(T)>);
    const _: () = {
        impl<T> core::clone::Clone for Pallet<T> {
            fn clone(&self) -> Self {
                Self(core::clone::Clone::clone(&self.0))
            }
        }
    };
    const _: () = {
        impl<T> core::cmp::Eq for Pallet<T> {}
    };
    const _: () = {
        impl<T> core::cmp::PartialEq for Pallet<T> {
            fn eq(&self, other: &Self) -> bool {
                true && self.0 == other.0
            }
        }
    };
    const _: () = {
        impl<T> core::fmt::Debug for Pallet<T> {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                fmt.debug_tuple("Pallet").field(&self.0).finish()
            }
        }
    };
    #[allow(type_alias_bounds)]
    pub(super) type Recoverable<T: Config> = StorageMap<
        _GeneratedPrefixForStorageRecoverable<T>,
        Blake2_128Concat,
        T::AccountId,
        Option<RecoveryConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>>,
        ValueQuery,
    >;
    #[allow(type_alias_bounds)]
    pub(super) type ActiveRecoveries<T: Config> = StorageDoubleMap<
        _GeneratedPrefixForStorageActiveRecoveries<T>,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        Option<ActiveRecovery<T::BlockNumber, BalanceOf<T>, T::AccountId>>,
        ValueQuery,
    >;
    #[allow(type_alias_bounds)]
    pub(super) type Proxy<T: Config> = StorageMap<
        _GeneratedPrefixForStorageProxy<T>,
        Blake2_128Concat,
        T::AccountId,
        Option<T::AccountId>,
        ValueQuery,
    >;
    impl<T: Config> Pallet<T> {
        /// Send a call through a recovered account.
        ///
        /// The dispatch origin for this call must be _Signed_ and registered to
        /// be able to make calls on behalf of the recovered account.
        ///
        /// Parameters:
        /// - `account`: The recovered account you want to make a call on-behalf-of.
        /// - `call`: The call you want to make with the recovered account.
        ///
        /// # <weight>
        /// - The weight of the `call` + 10,000.
        /// - One storage lookup to check account is recovered by `who`. O(1)
        /// # </weight>
        pub fn as_recovered(
            origin: OriginFor<T>,
            account: T::AccountId,
            call: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let target = Self::proxy(&who).ok_or(Error::<T>::NotAllowed)?;
            {
                if !(&target == &account) {
                    {
                        return Err(Error::<T>::NotAllowed.into());
                    };
                }
            };
            let _ = call
                .dispatch(frame_system::RawOrigin::Signed(account).into())
                .map(|_| ())
                .map_err(|e| e.error);
            Ok(().into())
        }
        /// Allow ROOT to bypass the xrecovery process and set an a rescuer account
        /// for a lost account directly.
        ///
        /// The dispatch origin for this call must be _ROOT_.
        ///
        /// Parameters:
        /// - `lost`: The "lost account" to be recovered.
        /// - `rescuer`: The "rescuer account" which can call as the lost account.
        ///
        /// # <weight>
        /// - One storage write O(1)
        /// - One event
        /// # </weight>
        pub fn set_recovered(
            origin: OriginFor<T>,
            lost: T::AccountId,
            rescuer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            <Proxy<T>>::insert(&rescuer, Some(&lost));
            Self::deposit_event(Event::AccountRecovered(lost, rescuer));
            Ok(().into())
        }
        /// Create a xrecovery configuration for your account. This makes your account recoverable.
        ///
        /// Payment: `ConfigDepositBase` + `FriendDepositFactor` * #_of_friends balance
        /// will be reserved for storing the xrecovery configuration. This deposit is returned
        /// in full when the user calls `remove_recovery`.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Parameters:
        /// - `friends`: A list of friends you trust to vouch for xrecovery attempts.
        ///   Should be ordered and contain no duplicate values.
        /// - `threshold`: The number of friends that must vouch for a xrecovery attempt
        ///   before the account can be recovered. Should be less than or equal to
        ///   the length of the list of friends.
        /// - `delay_period`: The number of blocks after a xrecovery attempt is initialized
        ///   that needs to pass before the account can be recovered.
        ///
        /// # <weight>
        /// - Key: F (len of friends)
        /// - One storage read to check that account is not already recoverable. O(1).
        /// - A check that the friends list is sorted and unique. O(F)
        /// - One currency reserve operation. O(X)
        /// - One storage write. O(1). Codec O(F).
        /// - One event.
        ///
        /// Total Complexity: O(F + X)
        /// # </weight>
        pub fn create_recovery(
            origin: OriginFor<T>,
            friends: Vec<T::AccountId>,
            threshold: u16,
            delay_period: T::BlockNumber,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            {
                if !!<Recoverable<T>>::contains_key(&who) {
                    {
                        return Err(Error::<T>::AlreadyRecoverable.into());
                    };
                }
            };
            {
                if !(threshold >= 1) {
                    {
                        return Err(Error::<T>::ZeroThreshold.into());
                    };
                }
            };
            {
                if !!friends.is_empty() {
                    {
                        return Err(Error::<T>::NotEnoughFriends.into());
                    };
                }
            };
            {
                if !(threshold as usize <= friends.len()) {
                    {
                        return Err(Error::<T>::NotEnoughFriends.into());
                    };
                }
            };
            let max_friends = T::MaxFriends::get() as usize;
            {
                if !(friends.len() <= max_friends) {
                    {
                        return Err(Error::<T>::MaxFriends.into());
                    };
                }
            };
            {
                if !Self::is_sorted_and_unique(&friends) {
                    {
                        return Err(Error::<T>::NotSorted.into());
                    };
                }
            };
            let friend_deposit = T::FriendDepositFactor::get()
                .checked_mul(&friends.len().saturated_into())
                .ok_or(Error::<T>::Overflow)?;
            let total_deposit = T::ConfigDepositBase::get()
                .checked_add(&friend_deposit)
                .ok_or(Error::<T>::Overflow)?;
            T::Currency::reserve(&who, total_deposit)?;
            let recovery_config = RecoveryConfig {
                delay_period,
                deposit: total_deposit,
                friends,
                threshold,
            };
            let call = <Recoverable<T>>::insert(&who, Some(recovery_config));
            Self::deposit_event(Event::RecoveryCreated(who));
            Ok(().into())
        }
        /// Initiate the process for recovering a recoverable account.
        ///
        /// Payment: `RecoveryDeposit` balance will be reserved for initiating the
        /// xrecovery process. This deposit will always be repatriated to the account
        /// trying to be recovered. See `close_recovery`.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Parameters:
        /// - `account`: The lost account that you want to recover. This account
        ///   needs to be recoverable (i.e. have a xrecovery configuration).
        ///
        /// # <weight>
        /// - One storage read to check that account is recoverable. O(F)
        /// - One storage read to check that this xrecovery process hasn't already started. O(1)
        /// - One currency reserve operation. O(X)
        /// - One storage read to get the current block number. O(1)
        /// - One storage write. O(1).
        /// - One event.
        ///
        /// Total Complexity: O(F + X)
        /// # </weight>
        pub fn initiate_recovery(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            {
                if !<Recoverable<T>>::contains_key(&account) {
                    {
                        return Err(Error::<T>::NotRecoverable.into());
                    };
                }
            };
            {
                if !!<ActiveRecoveries<T>>::contains_key(&account, &who) {
                    {
                        return Err(Error::<T>::AlreadyStarted.into());
                    };
                }
            };
            let recovery_deposit = T::RecoveryDeposit::get();
            T::Currency::reserve(&who, recovery_deposit)?;
            let recovery_status = ActiveRecovery {
                created: <system::Pallet<T>>::block_number(),
                deposit: recovery_deposit,
                friends: ::alloc::vec::Vec::new(),
            };
            <ActiveRecoveries<T>>::insert(&account, &who, Some(recovery_status));
            Self::deposit_event(Event::RecoveryInitiated(account, who));
            Ok(().into())
        }
        /// Allow a "friend" of a recoverable account to vouch for an active xrecovery
        /// process for that account.
        ///
        /// The dispatch origin for this call must be _Signed_ and must be a "friend"
        /// for the recoverable account.
        ///
        /// Parameters:
        /// - `lost`: The lost account that you want to recover.
        /// - `rescuer`: The account trying to rescue the lost account that you
        ///   want to vouch for.
        ///
        /// The combination of these two parameters must point to an active xrecovery
        /// process.
        ///
        /// # <weight>
        /// Key: F (len of friends in config), V (len of vouching friends)
        /// - One storage read to get the xrecovery configuration. O(1), Codec O(F)
        /// - One storage read to get the active xrecovery process. O(1), Codec O(V)
        /// - One binary search to confirm caller is a friend. O(logF)
        /// - One binary search to confirm caller has not already vouched. O(logV)
        /// - One storage write. O(1), Codec O(V).
        /// - One event.
        ///
        /// Total Complexity: O(F + logF + V + logV)
        /// # </weight>
        pub fn vouch_recovery(
            origin: OriginFor<T>,
            lost: T::AccountId,
            rescuer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let recovery_config = Self::recovery_config(&lost).ok_or(Error::<T>::NotRecoverable)?;
            let mut active_recovery =
                Self::active_recovery(&lost, &rescuer).ok_or(Error::<T>::NotStarted)?;
            {
                if !Self::is_friend(&recovery_config.friends, &who) {
                    {
                        return Err(Error::<T>::NotFriend.into());
                    };
                }
            };
            match active_recovery.friends.binary_search(&who) {
                Ok(_pos) => Err(Error::<T>::AlreadyVouched)?,
                Err(pos) => active_recovery.friends.insert(pos, who.clone()),
            }
            <ActiveRecoveries<T>>::insert(&lost, &rescuer, Some(active_recovery));
            Self::deposit_event(Event::RecoveryVouched(lost, rescuer, who));
            Ok(().into())
        }
        /// Allow a successful rescuer to claim their recovered account.
        ///
        /// The dispatch origin for this call must be _Signed_ and must be a "rescuer"
        /// who has successfully completed the account xrecovery process: collected
        /// `threshold` or more vouches, waited `delay_period` blocks since initiation.
        ///
        /// Parameters:
        /// - `account`: The lost account that you want to claim has been successfully
        ///   recovered by you.
        ///
        /// # <weight>
        /// Key: F (len of friends in config), V (len of vouching friends)
        /// - One storage read to get the xrecovery configuration. O(1), Codec O(F)
        /// - One storage read to get the active xrecovery process. O(1), Codec O(V)
        /// - One storage read to get the current block number. O(1)
        /// - One storage write. O(1), Codec O(V).
        /// - One event.
        ///
        /// Total Complexity: O(F + V)
        /// # </weight>
        pub fn claim_recovery(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let recovery_config =
                Self::recovery_config(&account).ok_or(Error::<T>::NotRecoverable)?;
            let active_recovery =
                Self::active_recovery(&account, &who).ok_or(Error::<T>::NotStarted)?;
            {
                if !!Proxy::<T>::contains_key(&who) {
                    {
                        return Err(Error::<T>::AlreadyProxy.into());
                    };
                }
            };
            let current_block_number = <system::Pallet<T>>::block_number();
            let recoverable_block_number = active_recovery
                .created
                .checked_add(&recovery_config.delay_period)
                .ok_or(Error::<T>::Overflow)?;
            {
                if !(recoverable_block_number <= current_block_number) {
                    {
                        return Err(Error::<T>::DelayPeriod.into());
                    };
                }
            };
            {
                if !(recovery_config.threshold as usize <= active_recovery.friends.len()) {
                    {
                        return Err(Error::<T>::Threshold.into());
                    };
                }
            };
            system::Pallet::<T>::inc_consumers(&who).map_err(|_| Error::<T>::BadState)?;
            Proxy::<T>::insert(&who, Some(&account));
            Self::deposit_event(Event::AccountRecovered(account, who));
            Ok(().into())
        }
        /// As the controller of a recoverable account, close an active xrecovery
        /// process for your account.
        ///
        /// Payment: By calling this function, the recoverable account will receive
        /// the xrecovery deposit `RecoveryDeposit` placed by the rescuer.
        ///
        /// The dispatch origin for this call must be _Signed_ and must be a
        /// recoverable account with an active xrecovery process for it.
        ///
        /// Parameters:
        /// - `rescuer`: The account trying to rescue this recoverable account.
        ///
        /// # <weight>
        /// Key: V (len of vouching friends)
        /// - One storage read/remove to get the active xrecovery process. O(1), Codec O(V)
        /// - One balance call to repatriate reserved. O(X)
        /// - One event.
        ///
        /// Total Complexity: O(V + X)
        /// # </weight>
        pub fn close_recovery(
            origin: OriginFor<T>,
            rescuer: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let active_recovery =
                <ActiveRecoveries<T>>::take(&who, &rescuer).ok_or(Error::<T>::NotStarted)?;
            let res = T::Currency::repatriate_reserved(
                &rescuer,
                &who,
                active_recovery.deposit,
                BalanceStatus::Free,
            );
            if true {
                if !res.is_ok() {
                    ::core::panicking::panic("assertion failed: res.is_ok()")
                };
            };
            Self::deposit_event(Event::RecoveryClosed(who, rescuer));
            Ok(().into())
        }
        /// Remove the xrecovery process for your account. Recovered accounts are still accessible.
        ///
        /// NOTE: The user must make sure to call `close_recovery` on all active
        /// xrecovery attempts before calling this function else it will fail.
        ///
        /// Payment: By calling this function the recoverable account will unreserve
        /// their xrecovery configuration deposit.
        /// (`ConfigDepositBase` + `FriendDepositFactor` * #_of_friends)
        ///
        /// The dispatch origin for this call must be _Signed_ and must be a
        /// recoverable account (i.e. has a xrecovery configuration).
        ///
        /// # <weight>
        /// Key: F (len of friends)
        /// - One storage read to get the prefix iterator for active recoveries. O(1)
        /// - One storage read/remove to get the xrecovery configuration. O(1), Codec O(F)
        /// - One balance call to unreserved. O(X)
        /// - One event.
        ///
        /// Total Complexity: O(F + X)
        /// # </weight>
        pub fn remove_recovery(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let mut active_recoveries = <ActiveRecoveries<T>>::iter_prefix_values(&who);
            {
                if !active_recoveries.next().is_none() {
                    {
                        return Err(Error::<T>::StillActive.into());
                    };
                }
            };
            let recovery_config = <Recoverable<T>>::take(&who).ok_or(Error::<T>::NotRecoverable)?;
            T::Currency::unreserve(&who, recovery_config.deposit);
            Self::deposit_event(Event::RecoveryRemoved(who));
            Ok(().into())
        }
        /// Cancel the ability to use `as_recovered` for `account`.
        ///
        /// The dispatch origin for this call must be _Signed_ and registered to
        /// be able to make calls on behalf of the recovered account.
        ///
        /// Parameters:
        /// - `account`: The recovered account you are able to call on-behalf-of.
        ///
        /// # <weight>
        /// - One storage mutation to check account is recovered by `who`. O(1)
        /// # </weight>
        pub fn cancel_recovered(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            {
                if !(Self::proxy(&who) == Some(account)) {
                    {
                        return Err(Error::<T>::NotAllowed.into());
                    };
                }
            };
            Proxy::<T>::remove(&who);
            system::Pallet::<T>::dec_consumers(&who);
            Ok(().into())
        }
    }
    impl<T: Config> Pallet<T> {
        fn send_to_litentry(origin: OriginFor<T>, account: T::AccountId) {
            let xcm_call = <T as Config>::Call::Balances(BalancesCall::transfer(1, 110));
        }
        /// Check that friends list is sorted and has no duplicates.
        fn is_sorted_and_unique(friends: &Vec<T::AccountId>) -> bool {
            friends.windows(2).all(|w| w[0] < w[1])
        }
        /// Check that a user is a friend in the friends list.
        fn is_friend(friends: &Vec<T::AccountId>, friend: &T::AccountId) -> bool {
            friends.binary_search(&friend).is_ok()
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn module_constants_metadata(
        ) -> &'static [frame_support::dispatch::ModuleConstantMetadata] {
            &[]
        }
    }
    impl<T: Config> frame_support::error::ModuleErrorMetadata for Pallet<T> {
        fn metadata() -> &'static [frame_support::error::ErrorMetadata] {
            <Error<T> as frame_support::error::ModuleErrorMetadata>::metadata()
        }
    }
    /// Type alias to `Pallet`, to be used by `construct_runtime`.
    ///
    /// Generated by `pallet` attribute macro.
    pub type Module<T> = Pallet<T>;
    impl<T: Config> frame_support::traits::GetPalletVersion for Pallet<T> {
        fn current_version() -> frame_support::traits::PalletVersion {
            frame_support::traits::PalletVersion {
                major: 0u16,
                minor: 0u8,
                patch: 1u8,
            }
        }
        fn storage_version() -> Option<frame_support::traits::PalletVersion> {
            let key = frame_support::traits::PalletVersion::storage_key::<
                <T as frame_system::Config>::PalletInfo,
                Self,
            >()
            .expect("Every active pallet has a name in the runtime; qed");
            frame_support::storage::unhashed::get(&key)
        }
    }
    impl<T: Config> frame_support::traits::OnGenesis for Pallet<T> {
        fn on_genesis() {
            frame_support::traits::PalletVersion {
                major: 0u16,
                minor: 0u8,
                patch: 1u8,
            }
            .put_into_storage::<<T as frame_system::Config>::PalletInfo, Self>();
        }
    }
    #[allow(non_camel_case_types)]
    pub enum Call<T: Config> {
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(
            frame_support::sp_std::marker::PhantomData<(T,)>,
            frame_support::Never,
        ),
        as_recovered(T::AccountId, Box<<T as Config>::Call>),
        set_recovered(T::AccountId, T::AccountId),
        create_recovery(Vec<T::AccountId>, u16, T::BlockNumber),
        initiate_recovery(T::AccountId),
        vouch_recovery(T::AccountId, T::AccountId),
        claim_recovery(T::AccountId),
        close_recovery(T::AccountId),
        remove_recovery(),
        cancel_recovered(T::AccountId),
    }
    const _: () = {
        impl<T: Config> core::fmt::Debug for Call<T> {
            fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
                match *self {
                    Self::__Ignore(ref _0, ref _1) => fmt
                        .debug_tuple("Call::__Ignore")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::as_recovered(ref _0, ref _1) => fmt
                        .debug_tuple("Call::as_recovered")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::set_recovered(ref _0, ref _1) => fmt
                        .debug_tuple("Call::set_recovered")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::create_recovery(ref _0, ref _1, ref _2) => fmt
                        .debug_tuple("Call::create_recovery")
                        .field(&_0)
                        .field(&_1)
                        .field(&_2)
                        .finish(),
                    Self::initiate_recovery(ref _0) => fmt
                        .debug_tuple("Call::initiate_recovery")
                        .field(&_0)
                        .finish(),
                    Self::vouch_recovery(ref _0, ref _1) => fmt
                        .debug_tuple("Call::vouch_recovery")
                        .field(&_0)
                        .field(&_1)
                        .finish(),
                    Self::claim_recovery(ref _0) => {
                        fmt.debug_tuple("Call::claim_recovery").field(&_0).finish()
                    }
                    Self::close_recovery(ref _0) => {
                        fmt.debug_tuple("Call::close_recovery").field(&_0).finish()
                    }
                    Self::remove_recovery() => fmt.debug_tuple("Call::remove_recovery").finish(),
                    Self::cancel_recovered(ref _0) => fmt
                        .debug_tuple("Call::cancel_recovered")
                        .field(&_0)
                        .finish(),
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::clone::Clone for Call<T> {
            fn clone(&self) -> Self {
                match self {
                    Self::__Ignore(ref _0, ref _1) => {
                        Self::__Ignore(core::clone::Clone::clone(_0), core::clone::Clone::clone(_1))
                    }
                    Self::as_recovered(ref _0, ref _1) => Self::as_recovered(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::set_recovered(ref _0, ref _1) => Self::set_recovered(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::create_recovery(ref _0, ref _1, ref _2) => Self::create_recovery(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                        core::clone::Clone::clone(_2),
                    ),
                    Self::initiate_recovery(ref _0) => {
                        Self::initiate_recovery(core::clone::Clone::clone(_0))
                    }
                    Self::vouch_recovery(ref _0, ref _1) => Self::vouch_recovery(
                        core::clone::Clone::clone(_0),
                        core::clone::Clone::clone(_1),
                    ),
                    Self::claim_recovery(ref _0) => {
                        Self::claim_recovery(core::clone::Clone::clone(_0))
                    }
                    Self::close_recovery(ref _0) => {
                        Self::close_recovery(core::clone::Clone::clone(_0))
                    }
                    Self::remove_recovery() => Self::remove_recovery(),
                    Self::cancel_recovered(ref _0) => {
                        Self::cancel_recovered(core::clone::Clone::clone(_0))
                    }
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> core::cmp::Eq for Call<T> {}
    };
    const _: () = {
        impl<T: Config> core::cmp::PartialEq for Call<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::__Ignore(_0, _1), Self::__Ignore(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::as_recovered(_0, _1), Self::as_recovered(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::set_recovered(_0, _1), Self::set_recovered(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (
                        Self::create_recovery(_0, _1, _2),
                        Self::create_recovery(_0_other, _1_other, _2_other),
                    ) => true && _0 == _0_other && _1 == _1_other && _2 == _2_other,
                    (Self::initiate_recovery(_0), Self::initiate_recovery(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (Self::vouch_recovery(_0, _1), Self::vouch_recovery(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::claim_recovery(_0), Self::claim_recovery(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (Self::close_recovery(_0), Self::close_recovery(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (Self::remove_recovery(), Self::remove_recovery()) => true,
                    (Self::cancel_recovered(_0), Self::cancel_recovered(_0_other)) => {
                        true && _0 == _0_other
                    }
                    (Self::__Ignore { .. }, Self::as_recovered { .. }) => false,
                    (Self::__Ignore { .. }, Self::set_recovered { .. }) => false,
                    (Self::__Ignore { .. }, Self::create_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::claim_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::close_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::remove_recovery { .. }) => false,
                    (Self::__Ignore { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::as_recovered { .. }, Self::__Ignore { .. }) => false,
                    (Self::as_recovered { .. }, Self::set_recovered { .. }) => false,
                    (Self::as_recovered { .. }, Self::create_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::claim_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::close_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::remove_recovery { .. }) => false,
                    (Self::as_recovered { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::set_recovered { .. }, Self::__Ignore { .. }) => false,
                    (Self::set_recovered { .. }, Self::as_recovered { .. }) => false,
                    (Self::set_recovered { .. }, Self::create_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::claim_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::close_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::remove_recovery { .. }) => false,
                    (Self::set_recovered { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::create_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::create_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::create_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::create_recovery { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::create_recovery { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::create_recovery { .. }, Self::claim_recovery { .. }) => false,
                    (Self::create_recovery { .. }, Self::close_recovery { .. }) => false,
                    (Self::create_recovery { .. }, Self::remove_recovery { .. }) => false,
                    (Self::create_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::create_recovery { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::claim_recovery { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::close_recovery { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::remove_recovery { .. }) => false,
                    (Self::initiate_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::create_recovery { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::claim_recovery { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::close_recovery { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::remove_recovery { .. }) => false,
                    (Self::vouch_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::claim_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::claim_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::claim_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::claim_recovery { .. }, Self::create_recovery { .. }) => false,
                    (Self::claim_recovery { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::claim_recovery { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::claim_recovery { .. }, Self::close_recovery { .. }) => false,
                    (Self::claim_recovery { .. }, Self::remove_recovery { .. }) => false,
                    (Self::claim_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::close_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::close_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::close_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::close_recovery { .. }, Self::create_recovery { .. }) => false,
                    (Self::close_recovery { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::close_recovery { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::close_recovery { .. }, Self::claim_recovery { .. }) => false,
                    (Self::close_recovery { .. }, Self::remove_recovery { .. }) => false,
                    (Self::close_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::remove_recovery { .. }, Self::__Ignore { .. }) => false,
                    (Self::remove_recovery { .. }, Self::as_recovered { .. }) => false,
                    (Self::remove_recovery { .. }, Self::set_recovered { .. }) => false,
                    (Self::remove_recovery { .. }, Self::create_recovery { .. }) => false,
                    (Self::remove_recovery { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::remove_recovery { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::remove_recovery { .. }, Self::claim_recovery { .. }) => false,
                    (Self::remove_recovery { .. }, Self::close_recovery { .. }) => false,
                    (Self::remove_recovery { .. }, Self::cancel_recovered { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::__Ignore { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::as_recovered { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::set_recovered { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::create_recovery { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::initiate_recovery { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::vouch_recovery { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::claim_recovery { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::close_recovery { .. }) => false,
                    (Self::cancel_recovered { .. }, Self::remove_recovery { .. }) => false,
                }
            }
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<T: Config> _parity_scale_codec::Encode for Call<T>
        where
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            Box<<T as Config>::Call>: _parity_scale_codec::Encode,
            Box<<T as Config>::Call>: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            Vec<T::AccountId>: _parity_scale_codec::Encode,
            Vec<T::AccountId>: _parity_scale_codec::Encode,
            T::BlockNumber: _parity_scale_codec::Encode,
            T::BlockNumber: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
        {
            fn encode_to<__CodecOutputEdqy: _parity_scale_codec::Output + ?Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Call::as_recovered(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(0usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Call::set_recovered(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(1usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Call::create_recovery(ref aa, ref ba, ref ca) => {
                        __codec_dest_edqy.push_byte(2usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ca, __codec_dest_edqy);
                    }
                    Call::initiate_recovery(ref aa) => {
                        __codec_dest_edqy.push_byte(3usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    Call::vouch_recovery(ref aa, ref ba) => {
                        __codec_dest_edqy.push_byte(4usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                        _parity_scale_codec::Encode::encode_to(ba, __codec_dest_edqy);
                    }
                    Call::claim_recovery(ref aa) => {
                        __codec_dest_edqy.push_byte(5usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    Call::close_recovery(ref aa) => {
                        __codec_dest_edqy.push_byte(6usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    Call::remove_recovery() => {
                        __codec_dest_edqy.push_byte(7usize as u8);
                    }
                    Call::cancel_recovered(ref aa) => {
                        __codec_dest_edqy.push_byte(8usize as u8);
                        _parity_scale_codec::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => (),
                }
            }
        }
        impl<T: Config> _parity_scale_codec::EncodeLike for Call<T>
        where
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            Box<<T as Config>::Call>: _parity_scale_codec::Encode,
            Box<<T as Config>::Call>: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            Vec<T::AccountId>: _parity_scale_codec::Encode,
            Vec<T::AccountId>: _parity_scale_codec::Encode,
            T::BlockNumber: _parity_scale_codec::Encode,
            T::BlockNumber: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
            T::AccountId: _parity_scale_codec::Encode,
        {
        }
    };
    const _: () = {
        #[allow(unknown_lints)]
        #[allow(rust_2018_idioms)]
        extern crate codec as _parity_scale_codec;
        impl<T: Config> _parity_scale_codec::Decode for Call<T>
        where
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            Box<<T as Config>::Call>: _parity_scale_codec::Decode,
            Box<<T as Config>::Call>: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            Vec<T::AccountId>: _parity_scale_codec::Decode,
            Vec<T::AccountId>: _parity_scale_codec::Decode,
            T::BlockNumber: _parity_scale_codec::Decode,
            T::BlockNumber: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
            T::AccountId: _parity_scale_codec::Decode,
        {
            fn decode<__CodecInputEdqy: _parity_scale_codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> core::result::Result<Self, _parity_scale_codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| e.chain("Could not decode `Call`, failed to read variant byte"))?
                {
                    __codec_x_edqy if __codec_x_edqy == 0usize as u8 => {
                        Ok(Call::<T>::as_recovered(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::as_recovered.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy = < Box < < T as Config > :: Call > as _parity_scale_codec :: Decode > :: decode (__codec_input_edqy) ;
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::as_recovered.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 1usize as u8 => {
                        Ok(Call::<T>::set_recovered(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::set_recovered.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::set_recovered.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 2usize as u8 => {
                        Ok(Call::<T>::create_recovery(
                            {
                                let __codec_res_edqy =
                                    <Vec<T::AccountId> as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::create_recovery.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy = <u16 as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::create_recovery.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::BlockNumber as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::create_recovery.2`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 3usize as u8 => {
                        Ok(Call::<T>::initiate_recovery({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(
                                        e.chain("Could not decode `Call::initiate_recovery.0`")
                                    )
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    __codec_x_edqy if __codec_x_edqy == 4usize as u8 => {
                        Ok(Call::<T>::vouch_recovery(
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::vouch_recovery.0`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                            {
                                let __codec_res_edqy =
                                    <T::AccountId as _parity_scale_codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                match __codec_res_edqy {
                                    Err(e) => {
                                        return Err(
                                            e.chain("Could not decode `Call::vouch_recovery.1`")
                                        )
                                    }
                                    Ok(__codec_res_edqy) => __codec_res_edqy,
                                }
                            },
                        ))
                    }
                    __codec_x_edqy if __codec_x_edqy == 5usize as u8 => {
                        Ok(Call::<T>::claim_recovery({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(e.chain("Could not decode `Call::claim_recovery.0`"))
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    __codec_x_edqy if __codec_x_edqy == 6usize as u8 => {
                        Ok(Call::<T>::close_recovery({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(e.chain("Could not decode `Call::close_recovery.0`"))
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    __codec_x_edqy if __codec_x_edqy == 7usize as u8 => {
                        Ok(Call::<T>::remove_recovery())
                    }
                    __codec_x_edqy if __codec_x_edqy == 8usize as u8 => {
                        Ok(Call::<T>::cancel_recovered({
                            let __codec_res_edqy =
                                <T::AccountId as _parity_scale_codec::Decode>::decode(
                                    __codec_input_edqy,
                                );
                            match __codec_res_edqy {
                                Err(e) => {
                                    return Err(
                                        e.chain("Could not decode `Call::cancel_recovered.0`")
                                    )
                                }
                                Ok(__codec_res_edqy) => __codec_res_edqy,
                            }
                        }))
                    }
                    _ => Err("Could not decode `Call`, variant doesn\'t exist".into()),
                }
            }
        }
    };
    impl<T: Config> frame_support::dispatch::GetDispatchInfo for Call<T> {
        fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
            match *self {
                Self::as_recovered(ref account, ref call) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight = <dyn frame_support::dispatch::WeighData<(
                        &T::AccountId,
                        &Box<<T as Config>::Call>,
                    )>>::weigh_data(&base_weight, (account, call));
                    let class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &T::AccountId,
                        &Box<<T as Config>::Call>,
                    )>>::classify_dispatch(
                        &base_weight, (account, call)
                    );
                    let pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &T::AccountId,
                        &Box<<T as Config>::Call>,
                    )>>::pays_fee(&base_weight, (account, call));
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::set_recovered(ref lost, ref rescuer) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight = <dyn frame_support::dispatch::WeighData<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::weigh_data(&base_weight, (lost, rescuer));
                    let class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::classify_dispatch(
                        &base_weight, (lost, rescuer)
                    );
                    let pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::pays_fee(&base_weight, (lost, rescuer));
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::create_recovery(ref friends, ref threshold, ref delay_period) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight = <dyn frame_support::dispatch::WeighData<(
                        &Vec<T::AccountId>,
                        &u16,
                        &T::BlockNumber,
                    )>>::weigh_data(
                        &base_weight, (friends, threshold, delay_period)
                    );
                    let class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &Vec<T::AccountId>,
                        &u16,
                        &T::BlockNumber,
                    )>>::classify_dispatch(
                        &base_weight, (friends, threshold, delay_period)
                    );
                    let pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &Vec<T::AccountId>,
                        &u16,
                        &T::BlockNumber,
                    )>>::pays_fee(
                        &base_weight, (friends, threshold, delay_period)
                    );
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::initiate_recovery(ref account) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight =
                        <dyn frame_support::dispatch::WeighData<(&T::AccountId,)>>::weigh_data(
                            &base_weight,
                            (account,),
                        );
                    let class = < dyn frame_support :: dispatch :: ClassifyDispatch < (& T :: AccountId ,) > > :: classify_dispatch (& base_weight , (account ,)) ;
                    let pays_fee =
                        <dyn frame_support::dispatch::PaysFee<(&T::AccountId,)>>::pays_fee(
                            &base_weight,
                            (account,),
                        );
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::vouch_recovery(ref lost, ref rescuer) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight = <dyn frame_support::dispatch::WeighData<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::weigh_data(&base_weight, (lost, rescuer));
                    let class = <dyn frame_support::dispatch::ClassifyDispatch<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::classify_dispatch(
                        &base_weight, (lost, rescuer)
                    );
                    let pays_fee = <dyn frame_support::dispatch::PaysFee<(
                        &T::AccountId,
                        &T::AccountId,
                    )>>::pays_fee(&base_weight, (lost, rescuer));
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::claim_recovery(ref account) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight =
                        <dyn frame_support::dispatch::WeighData<(&T::AccountId,)>>::weigh_data(
                            &base_weight,
                            (account,),
                        );
                    let class = < dyn frame_support :: dispatch :: ClassifyDispatch < (& T :: AccountId ,) > > :: classify_dispatch (& base_weight , (account ,)) ;
                    let pays_fee =
                        <dyn frame_support::dispatch::PaysFee<(&T::AccountId,)>>::pays_fee(
                            &base_weight,
                            (account,),
                        );
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::close_recovery(ref rescuer) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight =
                        <dyn frame_support::dispatch::WeighData<(&T::AccountId,)>>::weigh_data(
                            &base_weight,
                            (rescuer,),
                        );
                    let class = < dyn frame_support :: dispatch :: ClassifyDispatch < (& T :: AccountId ,) > > :: classify_dispatch (& base_weight , (rescuer ,)) ;
                    let pays_fee =
                        <dyn frame_support::dispatch::PaysFee<(&T::AccountId,)>>::pays_fee(
                            &base_weight,
                            (rescuer,),
                        );
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::remove_recovery() => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight =
                        <dyn frame_support::dispatch::WeighData<()>>::weigh_data(&base_weight, ());
                    let class =
                        <dyn frame_support::dispatch::ClassifyDispatch<()>>::classify_dispatch(
                            &base_weight,
                            (),
                        );
                    let pays_fee =
                        <dyn frame_support::dispatch::PaysFee<()>>::pays_fee(&base_weight, ());
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::cancel_recovered(ref account) => {
                    let base_weight = <T as pallet::Config>::WeightInfo::asset_claim();
                    let weight =
                        <dyn frame_support::dispatch::WeighData<(&T::AccountId,)>>::weigh_data(
                            &base_weight,
                            (account,),
                        );
                    let class = < dyn frame_support :: dispatch :: ClassifyDispatch < (& T :: AccountId ,) > > :: classify_dispatch (& base_weight , (account ,)) ;
                    let pays_fee =
                        <dyn frame_support::dispatch::PaysFee<(&T::AccountId,)>>::pays_fee(
                            &base_weight,
                            (account,),
                        );
                    frame_support::dispatch::DispatchInfo {
                        weight,
                        class,
                        pays_fee,
                    }
                }
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["internal error: entered unreachable code: "],
                        &match (&"__Ignore cannot be used",) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
            }
        }
    }
    impl<T: Config> frame_support::dispatch::GetCallName for Call<T> {
        fn get_call_name(&self) -> &'static str {
            match *self {
                Self::as_recovered(..) => "as_recovered",
                Self::set_recovered(..) => "set_recovered",
                Self::create_recovery(..) => "create_recovery",
                Self::initiate_recovery(..) => "initiate_recovery",
                Self::vouch_recovery(..) => "vouch_recovery",
                Self::claim_recovery(..) => "claim_recovery",
                Self::close_recovery(..) => "close_recovery",
                Self::remove_recovery(..) => "remove_recovery",
                Self::cancel_recovered(..) => "cancel_recovered",
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["internal error: entered unreachable code: "],
                        &match (&"__PhantomItem cannot be used.",) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
            }
        }
        fn get_call_names() -> &'static [&'static str] {
            &[
                "as_recovered",
                "set_recovered",
                "create_recovery",
                "initiate_recovery",
                "vouch_recovery",
                "claim_recovery",
                "close_recovery",
                "remove_recovery",
                "cancel_recovered",
            ]
        }
    }
    impl<T: Config> frame_support::traits::UnfilteredDispatchable for Call<T> {
        type Origin = frame_system::pallet_prelude::OriginFor<T>;
        fn dispatch_bypass_filter(
            self,
            origin: Self::Origin,
        ) -> frame_support::dispatch::DispatchResultWithPostInfo {
            match self {
                Self::as_recovered(account, call) => {
                    <Pallet<T>>::as_recovered(origin, account, call)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::set_recovered(lost, rescuer) => {
                    <Pallet<T>>::set_recovered(origin, lost, rescuer)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::create_recovery(friends, threshold, delay_period) => {
                    <Pallet<T>>::create_recovery(origin, friends, threshold, delay_period)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::initiate_recovery(account) => <Pallet<T>>::initiate_recovery(origin, account)
                    .map(Into::into)
                    .map_err(Into::into),
                Self::vouch_recovery(lost, rescuer) => {
                    <Pallet<T>>::vouch_recovery(origin, lost, rescuer)
                        .map(Into::into)
                        .map_err(Into::into)
                }
                Self::claim_recovery(account) => <Pallet<T>>::claim_recovery(origin, account)
                    .map(Into::into)
                    .map_err(Into::into),
                Self::close_recovery(rescuer) => <Pallet<T>>::close_recovery(origin, rescuer)
                    .map(Into::into)
                    .map_err(Into::into),
                Self::remove_recovery() => <Pallet<T>>::remove_recovery(origin)
                    .map(Into::into)
                    .map_err(Into::into),
                Self::cancel_recovered(account) => <Pallet<T>>::cancel_recovered(origin, account)
                    .map(Into::into)
                    .map_err(Into::into),
                Self::__Ignore(_, _) => {
                    let _ = origin;
                    {
                        {
                            ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                                &["internal error: entered unreachable code: "],
                                &match (&"__PhantomItem cannot be used.",) {
                                    (arg0,) => [::core::fmt::ArgumentV1::new(
                                        arg0,
                                        ::core::fmt::Display::fmt,
                                    )],
                                },
                            ))
                        }
                    };
                }
            }
        }
    }
    impl<T: Config> frame_support::dispatch::Callable<T> for Pallet<T> {
        type Call = Call<T>;
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn call_functions() -> &'static [frame_support::dispatch::FunctionMetadata] {
            & [frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("as_recovered") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("account") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , } , frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("call") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("Box<<T as Config>::Call>") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Send a call through a recovered account." , "" , " The dispatch origin for this call must be _Signed_ and registered to" , " be able to make calls on behalf of the recovered account." , "" , " Parameters:" , " - `account`: The recovered account you want to make a call on-behalf-of." , " - `call`: The call you want to make with the recovered account." , "" , " # <weight>" , " - The weight of the `call` + 10,000." , " - One storage lookup to check account is recovered by `who`. O(1)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("set_recovered") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("lost") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , } , frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("rescuer") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Allow ROOT to bypass the xrecovery process and set an a rescuer account" , " for a lost account directly." , "" , " The dispatch origin for this call must be _ROOT_." , "" , " Parameters:" , " - `lost`: The \"lost account\" to be recovered." , " - `rescuer`: The \"rescuer account\" which can call as the lost account." , "" , " # <weight>" , " - One storage write O(1)" , " - One event" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("create_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("friends") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("Vec<T::AccountId>") , } , frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("threshold") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("u16") , } , frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("delay_period") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::BlockNumber") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Create a xrecovery configuration for your account. This makes your account recoverable." , "" , " Payment: `ConfigDepositBase` + `FriendDepositFactor` * #_of_friends balance" , " will be reserved for storing the xrecovery configuration. This deposit is returned" , " in full when the user calls `remove_recovery`." , "" , " The dispatch origin for this call must be _Signed_." , "" , " Parameters:" , " - `friends`: A list of friends you trust to vouch for xrecovery attempts." , "   Should be ordered and contain no duplicate values." , " - `threshold`: The number of friends that must vouch for a xrecovery attempt" , "   before the account can be recovered. Should be less than or equal to" , "   the length of the list of friends." , " - `delay_period`: The number of blocks after a xrecovery attempt is initialized" , "   that needs to pass before the account can be recovered." , "" , " # <weight>" , " - Key: F (len of friends)" , " - One storage read to check that account is not already recoverable. O(1)." , " - A check that the friends list is sorted and unique. O(F)" , " - One currency reserve operation. O(X)" , " - One storage write. O(1). Codec O(F)." , " - One event." , "" , " Total Complexity: O(F + X)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("initiate_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("account") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Initiate the process for recovering a recoverable account." , "" , " Payment: `RecoveryDeposit` balance will be reserved for initiating the" , " xrecovery process. This deposit will always be repatriated to the account" , " trying to be recovered. See `close_recovery`." , "" , " The dispatch origin for this call must be _Signed_." , "" , " Parameters:" , " - `account`: The lost account that you want to recover. This account" , "   needs to be recoverable (i.e. have a xrecovery configuration)." , "" , " # <weight>" , " - One storage read to check that account is recoverable. O(F)" , " - One storage read to check that this xrecovery process hasn\'t already started. O(1)" , " - One currency reserve operation. O(X)" , " - One storage read to get the current block number. O(1)" , " - One storage write. O(1)." , " - One event." , "" , " Total Complexity: O(F + X)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("vouch_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("lost") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , } , frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("rescuer") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Allow a \"friend\" of a recoverable account to vouch for an active xrecovery" , " process for that account." , "" , " The dispatch origin for this call must be _Signed_ and must be a \"friend\"" , " for the recoverable account." , "" , " Parameters:" , " - `lost`: The lost account that you want to recover." , " - `rescuer`: The account trying to rescue the lost account that you" , "   want to vouch for." , "" , " The combination of these two parameters must point to an active xrecovery" , " process." , "" , " # <weight>" , " Key: F (len of friends in config), V (len of vouching friends)" , " - One storage read to get the xrecovery configuration. O(1), Codec O(F)" , " - One storage read to get the active xrecovery process. O(1), Codec O(V)" , " - One binary search to confirm caller is a friend. O(logF)" , " - One binary search to confirm caller has not already vouched. O(logV)" , " - One storage write. O(1), Codec O(V)." , " - One event." , "" , " Total Complexity: O(F + logF + V + logV)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("claim_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("account") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Allow a successful rescuer to claim their recovered account." , "" , " The dispatch origin for this call must be _Signed_ and must be a \"rescuer\"" , " who has successfully completed the account xrecovery process: collected" , " `threshold` or more vouches, waited `delay_period` blocks since initiation." , "" , " Parameters:" , " - `account`: The lost account that you want to claim has been successfully" , "   recovered by you." , "" , " # <weight>" , " Key: F (len of friends in config), V (len of vouching friends)" , " - One storage read to get the xrecovery configuration. O(1), Codec O(F)" , " - One storage read to get the active xrecovery process. O(1), Codec O(V)" , " - One storage read to get the current block number. O(1)" , " - One storage write. O(1), Codec O(V)." , " - One event." , "" , " Total Complexity: O(F + V)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("close_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("rescuer") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" As the controller of a recoverable account, close an active xrecovery" , " process for your account." , "" , " Payment: By calling this function, the recoverable account will receive" , " the xrecovery deposit `RecoveryDeposit` placed by the rescuer." , "" , " The dispatch origin for this call must be _Signed_ and must be a" , " recoverable account with an active xrecovery process for it." , "" , " Parameters:" , " - `rescuer`: The account trying to rescue this recoverable account." , "" , " # <weight>" , " Key: V (len of vouching friends)" , " - One storage read/remove to get the active xrecovery process. O(1), Codec O(V)" , " - One balance call to repatriate reserved. O(X)" , " - One event." , "" , " Total Complexity: O(V + X)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("remove_recovery") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& []) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Remove the xrecovery process for your account. Recovered accounts are still accessible." , "" , " NOTE: The user must make sure to call `close_recovery` on all active" , " xrecovery attempts before calling this function else it will fail." , "" , " Payment: By calling this function the recoverable account will unreserve" , " their xrecovery configuration deposit." , " (`ConfigDepositBase` + `FriendDepositFactor` * #_of_friends)" , "" , " The dispatch origin for this call must be _Signed_ and must be a" , " recoverable account (i.e. has a xrecovery configuration)." , "" , " # <weight>" , " Key: F (len of friends)" , " - One storage read to get the prefix iterator for active recoveries. O(1)" , " - One storage read/remove to get the xrecovery configuration. O(1), Codec O(F)" , " - One balance call to unreserved. O(X)" , " - One event." , "" , " Total Complexity: O(F + X)" , " # </weight>"]) , } , frame_support :: dispatch :: FunctionMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("cancel_recovered") , arguments : frame_support :: dispatch :: DecodeDifferent :: Encode (& [frame_support :: dispatch :: FunctionArgumentMetadata { name : frame_support :: dispatch :: DecodeDifferent :: Encode ("account") , ty : frame_support :: dispatch :: DecodeDifferent :: Encode ("T::AccountId") , }]) , documentation : frame_support :: dispatch :: DecodeDifferent :: Encode (& [" Cancel the ability to use `as_recovered` for `account`." , "" , " The dispatch origin for this call must be _Signed_ and registered to" , " be able to make calls on behalf of the recovered account." , "" , " Parameters:" , " - `account`: The recovered account you are able to call on-behalf-of." , "" , " # <weight>" , " - One storage mutation to check account is recovered by `who`. O(1)" , " # </weight>"]) , }]
        }
    }
    impl<T: Config> frame_support::sp_std::fmt::Debug for Error<T> {
        fn fmt(
            &self,
            f: &mut frame_support::sp_std::fmt::Formatter<'_>,
        ) -> frame_support::sp_std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    impl<T: Config> Error<T> {
        pub fn as_u8(&self) -> u8 {
            match &self {
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["internal error: entered unreachable code: "],
                        &match (&"`__Ignore` can never be constructed",) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                Self::NotAllowed => 0usize as u8,
                Self::ZeroThreshold => 1usize as u8,
                Self::NotEnoughFriends => 2usize as u8,
                Self::MaxFriends => 3usize as u8,
                Self::NotSorted => 4usize as u8,
                Self::NotRecoverable => 5usize as u8,
                Self::AlreadyRecoverable => 6usize as u8,
                Self::AlreadyStarted => 7usize as u8,
                Self::NotStarted => 8usize as u8,
                Self::NotFriend => 9usize as u8,
                Self::DelayPeriod => 10usize as u8,
                Self::AlreadyVouched => 11usize as u8,
                Self::Threshold => 12usize as u8,
                Self::StillActive => 13usize as u8,
                Self::Overflow => 14usize as u8,
                Self::AlreadyProxy => 15usize as u8,
                Self::BadState => 16usize as u8,
            }
        }
        pub fn as_str(&self) -> &'static str {
            match &self {
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(
                        &["internal error: entered unreachable code: "],
                        &match (&"`__Ignore` can never be constructed",) {
                            (arg0,) => [::core::fmt::ArgumentV1::new(
                                arg0,
                                ::core::fmt::Display::fmt,
                            )],
                        },
                    ))
                }
                Self::NotAllowed => "NotAllowed",
                Self::ZeroThreshold => "ZeroThreshold",
                Self::NotEnoughFriends => "NotEnoughFriends",
                Self::MaxFriends => "MaxFriends",
                Self::NotSorted => "NotSorted",
                Self::NotRecoverable => "NotRecoverable",
                Self::AlreadyRecoverable => "AlreadyRecoverable",
                Self::AlreadyStarted => "AlreadyStarted",
                Self::NotStarted => "NotStarted",
                Self::NotFriend => "NotFriend",
                Self::DelayPeriod => "DelayPeriod",
                Self::AlreadyVouched => "AlreadyVouched",
                Self::Threshold => "Threshold",
                Self::StillActive => "StillActive",
                Self::Overflow => "Overflow",
                Self::AlreadyProxy => "AlreadyProxy",
                Self::BadState => "BadState",
            }
        }
    }
    impl<T: Config> From<Error<T>> for &'static str {
        fn from(err: Error<T>) -> &'static str {
            err.as_str()
        }
    }
    impl<T: Config> From<Error<T>> for frame_support::sp_runtime::DispatchError {
        fn from(err: Error<T>) -> Self {
            let index = < < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: index :: < Pallet < T > > () . expect ("Every active module has an index in the runtime; qed") as u8 ;
            frame_support::sp_runtime::DispatchError::Module {
                index,
                error: err.as_u8(),
                message: Some(err.as_str()),
            }
        }
    }
    impl<T: Config> frame_support::error::ModuleErrorMetadata for Error<T> {
        fn metadata() -> &'static [frame_support::error::ErrorMetadata] {
            &[
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotAllowed"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " User is not allowed to make a call on behalf of this account",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("ZeroThreshold"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " Threshold must be greater than zero",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotEnoughFriends"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " Friends list must be greater than zero and threshold",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("MaxFriends"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " Friends list must be less than max friends",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotSorted"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " Friends list must be sorted and free of duplicates",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotRecoverable"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " This account is not set up for xrecovery",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("AlreadyRecoverable"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " This account is already set up for xrecovery",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("AlreadyStarted"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " A xrecovery process has already started for this account",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotStarted"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " A xrecovery process has not started for this rescuer",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("NotFriend"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " This account is not a friend who can vouch",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("DelayPeriod"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " The friend must wait until the delay period to vouch for this xrecovery",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("AlreadyVouched"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " This user has already vouched for this xrecovery",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("Threshold"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " The threshold for recovering this account has not been met",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("StillActive"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " There are still active xrecovery attempts that need to be closed",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("Overflow"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " There was an overflow in a calculation",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("AlreadyProxy"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " This account is already set up for xrecovery",
                    ]),
                },
                frame_support::error::ErrorMetadata {
                    name: frame_support::error::DecodeDifferent::Encode("BadState"),
                    documentation: frame_support::error::DecodeDifferent::Encode(&[
                        " Some internal state is broken.",
                    ]),
                },
            ]
        }
    }
    impl<T: Config> Pallet<T> {
        pub(super) fn deposit_event(event: Event<T>) {
            let event = <<T as Config>::Event as From<Event<T>>>::from(event);
            let event =
                <<T as Config>::Event as Into<<T as frame_system::Config>::Event>>::into(event);
            <frame_system::Pallet<T>>::deposit_event(event)
        }
    }
    impl<T: Config> From<Event<T>> for () {
        fn from(_: Event<T>) -> () {
            ()
        }
    }
    impl<T: Config> Event<T> {
        #[allow(dead_code)]
        #[doc(hidden)]
        pub fn metadata() -> &'static [frame_support::event::EventMetadata] {
            & [frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("RecoveryCreated") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" A xrecovery process has been set up for an \\[account\\]."]) , } , frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("RecoveryInitiated") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId" , "AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" A xrecovery process has been initiated for lost account by rescuer account." , " \\[lost, rescuer\\]"]) , } , frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("RecoveryVouched") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId" , "AccountId" , "AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" A xrecovery process for lost account by rescuer account has been vouched for by sender." , " \\[lost, rescuer, sender\\]"]) , } , frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("RecoveryClosed") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId" , "AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" A xrecovery process for lost account by rescuer account has been closed." , " \\[lost, rescuer\\]"]) , } , frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("AccountRecovered") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId" , "AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" Lost account has been successfully recovered by rescuer account." , " \\[lost, rescuer\\]"]) , } , frame_support :: event :: EventMetadata { name : frame_support :: event :: DecodeDifferent :: Encode ("RecoveryRemoved") , arguments : frame_support :: event :: DecodeDifferent :: Encode (& ["AccountId"]) , documentation : frame_support :: event :: DecodeDifferent :: Encode (& [" A xrecovery process has been removed for an \\[account\\]."]) , }]
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn storage_metadata() -> frame_support::metadata::StorageMetadata {
            frame_support :: metadata :: StorageMetadata { prefix : frame_support :: metadata :: DecodeDifferent :: Encode (< < T as frame_system :: Config > :: PalletInfo as frame_support :: traits :: PalletInfo > :: name :: < Pallet < T > > () . expect ("Every active pallet has a name in the runtime; qed")) , entries : frame_support :: metadata :: DecodeDifferent :: Encode (& [frame_support :: metadata :: StorageEntryMetadata { name : frame_support :: metadata :: DecodeDifferent :: Encode (< Recoverable < T > as frame_support :: storage :: types :: StorageMapMetadata > :: NAME) , modifier : < Recoverable < T > as frame_support :: storage :: types :: StorageMapMetadata > :: MODIFIER , ty : frame_support :: metadata :: StorageEntryType :: Map { hasher : < Recoverable < T > as frame_support :: storage :: types :: StorageMapMetadata > :: HASHER , key : frame_support :: metadata :: DecodeDifferent :: Encode ("T::AccountId") , value : frame_support :: metadata :: DecodeDifferent :: Encode ("Option<RecoveryConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>\n>") , unused : false , } , default : frame_support :: metadata :: DecodeDifferent :: Encode (< Recoverable < T > as frame_support :: storage :: types :: StorageMapMetadata > :: DEFAULT) , documentation : frame_support :: metadata :: DecodeDifferent :: Encode (& []) , } , frame_support :: metadata :: StorageEntryMetadata { name : frame_support :: metadata :: DecodeDifferent :: Encode (< ActiveRecoveries < T > as frame_support :: storage :: types :: StorageDoubleMapMetadata > :: NAME) , modifier : < ActiveRecoveries < T > as frame_support :: storage :: types :: StorageDoubleMapMetadata > :: MODIFIER , ty : frame_support :: metadata :: StorageEntryType :: DoubleMap { hasher : < ActiveRecoveries < T > as frame_support :: storage :: types :: StorageDoubleMapMetadata > :: HASHER1 , key2_hasher : < ActiveRecoveries < T > as frame_support :: storage :: types :: StorageDoubleMapMetadata > :: HASHER2 , key1 : frame_support :: metadata :: DecodeDifferent :: Encode ("T::AccountId") , key2 : frame_support :: metadata :: DecodeDifferent :: Encode ("T::AccountId") , value : frame_support :: metadata :: DecodeDifferent :: Encode ("Option<ActiveRecovery<T::BlockNumber, BalanceOf<T>, T::AccountId>\n>") , } , default : frame_support :: metadata :: DecodeDifferent :: Encode (< ActiveRecoveries < T > as frame_support :: storage :: types :: StorageDoubleMapMetadata > :: DEFAULT) , documentation : frame_support :: metadata :: DecodeDifferent :: Encode (& []) , } , frame_support :: metadata :: StorageEntryMetadata { name : frame_support :: metadata :: DecodeDifferent :: Encode (< Proxy < T > as frame_support :: storage :: types :: StorageMapMetadata > :: NAME) , modifier : < Proxy < T > as frame_support :: storage :: types :: StorageMapMetadata > :: MODIFIER , ty : frame_support :: metadata :: StorageEntryType :: Map { hasher : < Proxy < T > as frame_support :: storage :: types :: StorageMapMetadata > :: HASHER , key : frame_support :: metadata :: DecodeDifferent :: Encode ("T::AccountId") , value : frame_support :: metadata :: DecodeDifferent :: Encode ("Option<T::AccountId>") , unused : false , } , default : frame_support :: metadata :: DecodeDifferent :: Encode (< Proxy < T > as frame_support :: storage :: types :: StorageMapMetadata > :: DEFAULT) , documentation : frame_support :: metadata :: DecodeDifferent :: Encode (& []) , }]) , }
        }
    }
    impl<T: Config> Pallet<T> {
        pub fn recovery_config<KArg>(
            k: KArg,
        ) -> Option<RecoveryConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>>
        where
            KArg: frame_support::codec::EncodeLike<T::AccountId>,
        {
            <Recoverable<T> as frame_support::storage::StorageMap<
                T::AccountId,
                Option<RecoveryConfig<T::BlockNumber, BalanceOf<T>, T::AccountId>>,
            >>::get(k)
        }
    }
    impl<T: Config> Pallet<T> {
        pub fn active_recovery<KArg1, KArg2>(
            k1: KArg1,
            k2: KArg2,
        ) -> Option<ActiveRecovery<T::BlockNumber, BalanceOf<T>, T::AccountId>>
        where
            KArg1: frame_support::codec::EncodeLike<T::AccountId>,
            KArg2: frame_support::codec::EncodeLike<T::AccountId>,
        {
            <ActiveRecoveries<T> as frame_support::storage::StorageDoubleMap<
                T::AccountId,
                T::AccountId,
                Option<ActiveRecovery<T::BlockNumber, BalanceOf<T>, T::AccountId>>,
            >>::get(k1, k2)
        }
    }
    impl<T: Config> Pallet<T> {
        pub fn proxy<KArg>(k: KArg) -> Option<T::AccountId>
        where
            KArg: frame_support::codec::EncodeLike<T::AccountId>,
        {
            < Proxy < T > as frame_support :: storage :: StorageMap < T :: AccountId , Option < T :: AccountId > > > :: get (k)
        }
    }
    pub(super) struct _GeneratedPrefixForStorageRecoverable<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
        for _GeneratedPrefixForStorageRecoverable<T>
    {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
            .expect("Every active pallet has a name in the runtime; qed")
        }
        const STORAGE_PREFIX: &'static str = "Recoverable";
    }
    pub(super) struct _GeneratedPrefixForStorageActiveRecoveries<T>(
        core::marker::PhantomData<(T,)>,
    );
    impl<T: Config> frame_support::traits::StorageInstance
        for _GeneratedPrefixForStorageActiveRecoveries<T>
    {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
            .expect("Every active pallet has a name in the runtime; qed")
        }
        const STORAGE_PREFIX: &'static str = "ActiveRecoveries";
    }
    pub(super) struct _GeneratedPrefixForStorageProxy<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance for _GeneratedPrefixForStorageProxy<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
            .expect("Every active pallet has a name in the runtime; qed")
        }
        const STORAGE_PREFIX: &'static str = "Proxy";
    }
    /// Hidden instance generated to be internally used when module is used without
    /// instance.
    #[doc(hidden)]
    pub type __InherentHiddenInstance = ();
    pub(super) trait Store {
        type Recoverable;
        type ActiveRecoveries;
        type Proxy;
    }
    impl<T: Config> Store for Pallet<T> {
        type Recoverable = Recoverable<T>;
        type ActiveRecoveries = ActiveRecoveries<T>;
        type Proxy = Proxy<T>;
    }
    impl<T: Config> frame_support::traits::OnFinalize<<T as frame_system::Config>::BlockNumber>
        for Pallet<T>
    {
        fn on_finalize(n: <T as frame_system::Config>::BlockNumber) {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_finalize (n)
        }
    }
    impl<T: Config> frame_support::traits::OnInitialize<<T as frame_system::Config>::BlockNumber>
        for Pallet<T>
    {
        fn on_initialize(
            n: <T as frame_system::Config>::BlockNumber,
        ) -> frame_support::weights::Weight {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: on_initialize (n)
        }
    }
    impl<T: Config> frame_support::traits::OnRuntimeUpgrade for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let result = <Self as frame_support::traits::Hooks<
                <T as frame_system::Config>::BlockNumber,
            >>::on_runtime_upgrade();
            frame_support::traits::PalletVersion {
                major: 0u16,
                minor: 0u8,
                patch: 1u8,
            }
            .put_into_storage::<<T as frame_system::Config>::PalletInfo, Self>();
            let additional_write =
                <<T as frame_system::Config>::DbWeight as frame_support::traits::Get<_>>::get()
                    .writes(1);
            result.saturating_add(additional_write)
        }
    }
    impl<T: Config> frame_support::traits::OffchainWorker<<T as frame_system::Config>::BlockNumber>
        for Pallet<T>
    {
        fn offchain_worker(n: <T as frame_system::Config>::BlockNumber) {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: offchain_worker (n)
        }
    }
    impl<T: Config> frame_support::traits::IntegrityTest for Pallet<T> {
        fn integrity_test() {
            < Self as frame_support :: traits :: Hooks < < T as frame_system :: Config > :: BlockNumber > > :: integrity_test ()
        }
    }
}
