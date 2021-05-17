#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use rstd::vec::Vec;
use sp_core::{RuntimeDebug, H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use xcm::v0::Junction;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryRegisterToLitentryCall {
    call_index: [u8; 2],
    module_id: u8,
    set_proxy_method_id: u8,
}

impl XrecoveryRegisterToLitentryCall {
    pub fn new(pallet_index: u8, call_index: u8, module_id: u8, set_proxy_method_id: u8) 
    -> Self {
        XrecoveryRegisterToLitentryCall {
            call_index: [pallet_index, call_index],
            module_id: module_id,
            set_proxy_method_id: set_proxy_method_id,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryCreateRecoveryCall {
    call_index: [u8; 2],
    account: Vec<u8>,
    friends: Vec<u8>,
    threshold: u16,
    delay_period: u32,
}

impl XrecoveryCreateRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, account: Vec<u8>, friends: Vec<u8>, threshold: u16, delay_period: u32) 
    -> Self {
        XrecoveryCreateRecoveryCall {
            call_index: [pallet_index, call_index],
            account: account,
            friends: friends,
            threshold: threshold,
            delay_period: delay_period,
        }
    }
}


#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryInitiateRecoveryCall {
    call_index: [u8; 2],
    account: Vec<u8>,
}

impl XrecoveryInitiateRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, account: Vec<u8>) 
    -> Self {
        XrecoveryInitiateRecoveryCall {
            call_index: [pallet_index, call_index],
            account: account,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryVouchRecoveryCall {
    call_index: [u8; 2],
    friend: Vec<u8>,
    lost: Vec<u8>,
    rescuer: Vec<u8>,

}

impl XrecoveryVouchRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, friend: Vec<u8>, lost: Vec<u8>, rescuer: Vec<u8>) 
    -> Self {
        XrecoveryVouchRecoveryCall {
            call_index: [pallet_index, call_index],
            friend: friend,
            lost: lost,
            rescuer: rescuer,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryClaimRecoveryCall {
    call_index: [u8; 2],
    account: Vec<u8>,

}

impl XrecoveryClaimRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, account: Vec<u8>) 
    -> Self {
        XrecoveryClaimRecoveryCall {
            call_index: [pallet_index, call_index],
            account: account,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryRemoveRecoveryCall {
    call_index: [u8; 2],
    account: Vec<u8>,

}

impl XrecoveryRemoveRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, account: Vec<u8>) 
    -> Self {
        XrecoveryRemoveRecoveryCall {
            call_index: [pallet_index, call_index],
            account: account,
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryCancelRecoveryCall {
    call_index: [u8; 2],
    account: Vec<u8>,

}

impl XrecoveryCancelRecoveryCall {
    pub fn new(pallet_index: u8, call_index: u8, account: Vec<u8>) 
    -> Self {
        XrecoveryCancelRecoveryCall {
            call_index: [pallet_index, call_index],
            account: account,
        }
    }
}