use codec::{Decode, Encode};
use rstd::vec::Vec;
use sp_core::{RuntimeDebug, H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use xcm::v0::Junction;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct XrecoveryCall {
    call_index: [u8; 2],
    request: OpaqueRequest,
    requested_response: u8,
    sender_pallet_index: u8,
}

pub type OpaqueRequest = Vec<u8>;

