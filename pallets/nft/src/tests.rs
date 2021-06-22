use crate::{mock::*};
use crate::{mock::Event};
use super::*;
use frame_support::{assert_ok, assert_noop};
use sp_runtime::AccountId32;

#[test]
fn test_issue_and_mint_eth() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(100),
		));

        // mint some NFTs
        assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
            other_account.clone(),
            0,
			CID::default(),
			2,
        ));	
    })
}

#[test]
fn test_issue_and_claim_eth() {
	new_test_ext().execute_with(|| {

    // account id of Alice 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		let alice_account: AccountId32 = AccountId32::from([0xd4,0x35,0x93,0xc7,0x15,0xfd,0xd3,0x1c,0x61,0x14,0x1a,0xbd,0x04,0xa9,0x9f,0xd6,0x82,0x2c,0x85,0x58,0x85,0x4c,0xcd,0xe3,0x9a,0x56,0x84,0xe7,0xa5,0x6d,0xa2,0x7d]);

    // root is 0x784756769ecc8bdd4b2ac239c57c1fd56a20e272c88b37131ef84e1541d35c3c
    let merkle_root = vec![0x78,0x47,0x56,0x76,0x9e,0xcc,0x8b,0xdd,0x4b,0x2a,0xc2,0x39,0xc5,0x7c,0x1f,0xd5,0x6a,0x20,0xe2,0x72,0xc8,0x8b,0x37,0x13,0x1e,0xf8,0x4e,0x15,0x41,0xd3,0x5c,0x3c];

    // proof of alice is 0xf4ee0d453c09d8b28507010935ffbc9ccc41044a4e7237405dbc1313f9ad140b
    let alice_proof = vec![[0xf4u8,0xeeu8,0x0du8,0x45u8,0x3cu8,0x09u8,0xd8u8,0xb2u8,0x85u8,0x07u8,0x01u8,0x09u8,0x35u8,0xffu8,0xbcu8,0x9cu8,0xccu8,0x41u8,0x04u8,0x4au8,0x4eu8,0x72u8,0x37u8,0x40u8,0x5du8,0xbcu8,0x13u8,0x13u8,0xf9u8,0xadu8,0x14u8,0x0bu8]];

    // issue a claim class
    assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			merkle_root.clone(), 
			Properties::default(), 
			None,
			None,
			ClassType::Claim(merkle_root),
		));

    //assert_eq!(
    //  events(),
    //  [
    //    Event::nft(crate::Event::CreatedClass(account.clone(), 0)),
    //  ]
    //);

    // claim with proof
    assert_ok!(Nft::claim(
			Origin::signed(alice_account.clone()),
      0,
      0,
      alice_proof,
      ));

    })
}

#[test]
fn test_issue_and_merge_eth() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		// issue basic NFTs
        assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(10),
		));

        assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Simple(10),
		));

		// mint some NFTs
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			1,
		));	

		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			1,
			CID::default(),
			1,
		));

		// issue advanced NFTs
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(), 
			Properties::default(), 
			None,
			None,
			ClassType::Merge(0, 1, false),
		));

        // claim with proof
        assert_ok!(Nft::merge(
			Origin::signed(other_account.clone()),
            2,
            (0, 0),
			(1, 0),
        ));
    })
}