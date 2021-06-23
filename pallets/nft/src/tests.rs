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

    // root is 0xc6529c59808f77135f28ef70b65356ddd583d934da3dba95bce23f49ea27258d 
    let merkle_root = vec![0xc6,0x52,0x9c,0x59,0x80,0x8f,0x77,0x13,0x5f,0x28,0xef,0x70,0xb6,0x53,0x56,0xdd,0xd5,0x83,0xd9,0x34,0xda,0x3d,0xba,0x95,0xbc,0xe2,0x3f,0x49,0xea,0x27,0x25,0x8d];

    // proof of alice is 0x3fa539b400a4c2de2822425972ae67b752dea0cf1bc8f37a48143f84e72b1d19 
    let alice_proof = vec![[0x3fu8,0xa5u8,0x39u8,0xb4u8,0x00u8,0xa4u8,0xc2u8,0xdeu8,0x28u8,0x22u8,0x42u8,0x59u8,0x72u8,0xaeu8,0x67u8,0xb7u8,0x52u8,0xdeu8,0xa0u8,0xcfu8,0x1bu8,0xc8u8,0xf3u8,0x7au8,0x48u8,0x14u8,0x3fu8,0x84u8,0xe7u8,0x2bu8,0x1du8,0x19u8]];

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

    // alice claims with random proof
    assert_noop!(Nft::claim(
			Origin::signed(alice_account.clone()),
      0,
      0,
      vec![[0u8; 32]],
      ),
      NftError::UserNotInClaimList
    );

    // alice claims with alice's proof
    assert_ok!(Nft::claim(
			Origin::signed(alice_account.clone()),
      0,
      0,
      alice_proof.clone(),
      ));

    // alice claims again
    assert_noop!(Nft::claim(
			Origin::signed(alice_account.clone()),
      0,
      0,
      alice_proof,
      ),
      NftError::TokenAlreadyClaimed
    );


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