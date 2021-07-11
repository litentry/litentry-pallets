#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, account};
use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks!{

    asset_claim {
        let caller = account("caller", 0, 0);
        
    }: asset_claim(RawOrigin::Signed(caller))

    submit_balance {
        let caller = account("caller", 0, 0);
        let account_id = account("Alice", 0, 0);
        <ClaimAccountIndex<T>>::insert(&account_id, Some(0_u32));
        let block_number = 1_u32;
        let data_source = urls::DataSource::EthEtherScan;
        let balance = 0_u128;
        
    }: submit_balance(RawOrigin::Signed(caller), account_id, block_number.into(), data_source.into(), balance)

    // benchmark with 100 records in CommitAccountBalance
    dummy {
        let caller: T::AccountId = account("caller", 0, 0);

        let block_number = 4_u32;
        let data_source = crate::urls::DataSource::EthEtherScan;

        for index in 1..100 {
            let sender: T::AccountId = account("sender", index, index);
            let balance = index as u128;
            let query_key: crate::QueryKey::<T::AccountId> = crate::QueryKey{
                account: sender.clone(),
                data_source: data_source};
    
            CommitAccountBalance::<T>::insert(&sender.clone(), &query_key, Some(balance));
        }

    }: dummy(RawOrigin::Signed(caller), block_number.into())
}

