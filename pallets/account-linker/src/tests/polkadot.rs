use crate::{mock::*};

use frame_support::assert_ok;
use sp_runtime::AccountId32;

#[test]
fn test_insert_polkadot_address() {
	new_test_ext().execute_with(|| {

        run_to_block(1);

        let origin: AccountId32 = AccountId32::from([0u8; 32]);
        let target: AccountId32 = AccountId32::from([1u8; 32]);

        assert_ok!(AccountLinker::link_polkadot(
            Origin::signed(origin.clone()),
            target.clone(),
            0
        ));

        assert_eq!(AccountLinker::polkadot_pending(&origin), (target.clone(), 0));

        assert_ok!(AccountLinker::accept_polkadot(
            Origin::signed(target.clone()),
            origin.clone()
        ));

        assert_eq!(
                events(),
                [
                    Event::account_linker(crate::Event::PolkadotAddressLinked(origin.clone(), target.clone())),
                ]
            );

		assert_eq!(AccountLinker::polkadot_addresses(&origin), vec![target]);
	});
}
