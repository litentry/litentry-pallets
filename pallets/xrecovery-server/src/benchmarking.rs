#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, account};
use frame_system::RawOrigin;
use sp_std::prelude::*;
use mock::{Call, BalancesCall,};

benchmarks!{

    as_recovered {
        let caller = account("caller", 0, 0);
		let call = Box::new(Call::Balances(BalancesCall::transfer(1, 110)));

    }: as_recovered(RawOrigin::Signed(caller), call)

//     submit_balance {
//         let caller = account("caller", 0, 0);
//         let account_id = account("Alice", 0, 0);
//         <ClaimAccountIndex<T>>::insert(&account_id, Some(0_u32));
//         let block_number = 1_u32;
//         let data_source = urls::DataSource::EthEtherScan;
//         let balance = 0_u128;
        
//     }: submit_balance(RawOrigin::Signed(caller), account_id, block_number.into(), data_source.into(), balance)
}

