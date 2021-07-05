#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, account};
use frame_system::RawOrigin;
use sp_std::prelude::*;
const SEED: u32 = 0;

benchmarks!{
    link_eth {
        let caller = account("caller", 0, 0);
        let account_id: T::AccountId = account("Alice", 0, SEED);
        let index: u32 = 0;
        let addr_expected: EthAddress = [129, 200, 95, 3, 120, 182, 178, 20, 15, 87, 86, 247, 198, 176, 71, 171, 101, 141, 253, 146];
        let expiring_block_number: u32 = 99999;
        let sig: Signature = [180, 104, 165, 108, 138, 215, 68, 214, 201, 125, 249, 69, 171, 96, 131, 77, 207, 50, 81, 87, 235, 49, 49, 16, 42, 59, 71, 107, 145, 216, 104, 51, 69, 8, 245, 57, 48, 110, 199, 38, 153, 47, 101, 78, 28, 228, 43, 113, 219, 80, 192, 119, 199, 67, 166, 118, 123, 71, 47, 167, 72, 203, 15, 105, 28];
    }:  link_eth(RawOrigin::Signed(caller), account_id.clone(), index, addr_expected, expiring_block_number.into(), sig)

    link_btc {
        let caller = account("caller", 0, 0);
        let account_id: T::AccountId = account("Alice", 0, SEED);
        let index: u32 = 0;
        let addr_expected = vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
        let expiring_block_number: u32 = 99999;
        let sig: Signature = [154, 132, 176, 211, 74, 112, 5, 170, 223, 234, 71, 130, 181, 88, 38, 174, 210, 132, 130, 2, 121, 64, 174, 205, 23, 117, 3, 22, 228, 221, 17, 48, 16, 252, 188, 138, 223, 134, 212, 153, 138, 60, 232, 216, 187, 175, 250, 159, 110, 153, 23, 181, 169, 126, 170, 131, 167, 97, 248, 55, 225, 71, 62, 167, 0];
    }:  link_btc(RawOrigin::Signed(caller), account_id.clone(), index, addr_expected, expiring_block_number.into(), sig)
}
