use super::*;
use crate::mock::Event;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;

#[test]
fn test_issue_and_mint_eth() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		let _ = Balances::deposit_creating(&account, (CREATION_FEE + 10).into());

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
		let alice_account: AccountId32 = AccountId32::from([
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		]);

		// root is 0xa8a5ec29a3df3c5a8aa6fd2935d2414cf0ce4f748a13bb2833214c3b94a6d3b3
		let merkle_root = [
			0xa8, 0xa5, 0xec, 0x29, 0xa3, 0xdf, 0x3c, 0x5a, 0x8a, 0xa6, 0xfd, 0x29, 0x35, 0xd2,
			0x41, 0x4c, 0xf0, 0xce, 0x4f, 0x74, 0x8a, 0x13, 0xbb, 0x28, 0x33, 0x21, 0x4c, 0x3b,
			0x94, 0xa6, 0xd3, 0xb3,
		];

		// proof of alice is 0x5182a73e48bd6e814d0c2b41672d9cb8c87c4221b55bc08e0943198e90caad1f
		let alice_proof = vec![[
			0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
			0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
			0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
		]];

		run_to_block(1);

		let _ = Balances::deposit_creating(&alice_account, (CREATION_FEE + 10).into());

		// issue a claim class
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties::default(),
			None,
			None,
			ClassType::Claim(merkle_root),
		));

		// assert_eq!(
		// 	events(),
		// 	[Event::Nft(crate::Event::CreatedClass(
		// 		alice_account.clone(),
		// 		0
		// 	)),]
		// );

		// alice claims with random proof
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, vec![[0u8; 32]],),
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
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof,),
			NftError::TokenAlreadyClaimed
		);
	})
}

#[test]
fn test_issue_and_merge_eth() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		let _ = Balances::deposit_creating(&account, (3 * CREATION_FEE + 10).into());

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
