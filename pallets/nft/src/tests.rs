use crate::{mock::*};
use super::*;
use frame_support::{assert_ok, assert_noop};


#[test]
fn test_issue_and_mint_eth() {
	new_test_ext().execute_with(|| {

		assert_ok!(Nft::create_class(
			Origin::signed(1),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(100),
		));

        // mint some NFTs
        assert_ok!(Nft::mint(
			Origin::signed(1),
            2,
            0,
			CID::default(),
			2,
        ));	
    })
}

#[test]
fn test_issue_and_claim_eth() {
	new_test_ext().execute_with(|| {

        // issue a claim class
        assert_ok!(Nft::create_class(
			Origin::signed(1),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Claim(CID::default()),
		));

        // assert_eq!(
		// 	events(),
		// 	[
		// 		Event::nft(crate::Event::CreatedClass(account.clone(), 0)),
		// 	]
		// );

        // claim with proof
        assert_ok!(Nft::claim(
			Origin::signed(1),
            0,
            u32::default(),
        ));

    })
}

#[test]
fn test_issue_and_merge_eth() {
	new_test_ext().execute_with(|| {

		// issue basic NFTs
        assert_ok!(Nft::create_class(
			Origin::signed(1),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(10),
		));

        assert_ok!(Nft::create_class(
			Origin::signed(1),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(10),
		));

		// mint some NFTs
		assert_ok!(Nft::mint(
			Origin::signed(1),
			2,
			0,
			CID::default(),
			1,
		));	

		assert_ok!(Nft::mint(
			Origin::signed(1),
			2,
			1,
			CID::default(),
			1,
		));

		// issue advanced NFTs
		assert_ok!(Nft::create_class(
			Origin::signed(1),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Merge(0, 1, false),
		));

        // claim with proof
        assert_ok!(Nft::merge(
			Origin::signed(2),
            2,
            (0, 0),
			(1, 0),
        ));
    })
}