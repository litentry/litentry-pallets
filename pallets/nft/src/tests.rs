// #![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};
use frame_system::Event as SystemEvent;
// use frame_system::AccountId;
use pallet_balances::Event as BalanceEvent;
use sp_runtime::AccountId32;

#[test]
fn test_simple_type() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		let _ = Balances::deposit_creating(&account, (CREATION_FEE + 10).into());
		let _ = Balances::deposit_creating(&other_account, (CREATION_FEE + 10).into());
		
		run_to_block(1);
		assert_eq!(System::block_number(), 1);

		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			// Properties(enumflags2::BitFlags::from(ClassProperty::Transferable)),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(100),
		));

		events();

		// mint some NFTs
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			2,
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::MintedToken(account.clone(), other_account.clone(), 0, 2)),]
		);

		// transfer some NFTs
		assert_ok!(Nft::transfer(
			Origin::signed(other_account.clone()),
			account.clone(),
			(0,1),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::TransferredToken(other_account.clone(), account.clone(), 0, 1)),]
		);

		// burn some NFTs
		assert_ok!(Nft::burn(
			Origin::signed(account.clone()),
			(0,1),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::BurnedToken(account.clone(), 0, 1)),]
		);


		//??Wonder if this will be pub/private issue
		assert_eq!(Nft::balance(&other_account), 1);
		assert_eq!(Nft::owner((0,0)).unwrap(), other_account);
		assert_ok!(Nft::do_transfer(
			&other_account,
			&account,
			(0,0),
		));
		assert_eq!(Nft::owner((0,0)).unwrap(), account);
	})
}

#[test]
fn test_claim_type() {
	new_test_ext().execute_with(|| {
		// account id of Alice 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d ,claimed index 0: 00 00
		let alice_account: AccountId32 = AccountId32::from([
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		]);

		// root is 0x0c67caf461290cd463e535213f996e32736e65a2063783fde5036b71396dfb0c
		let merkle_root = [
			0x0cu8, 0x67u8, 0xcau8, 0xf4u8, 0x61u8, 0x29u8, 0x0cu8, 0xd4u8, 0x63u8, 0xe5u8, 0x35u8, 
			0x21u8, 0x3fu8, 0x99u8, 0x6eu8, 0x32u8, 0x73u8, 0x6eu8, 0x65u8, 0xa2u8, 0x06u8, 0x37u8, 
			0x83u8, 0xfdu8, 0xe5u8, 0x03u8, 0x6bu8, 0x71u8, 0x39u8, 0x6du8, 0xfbu8, 0x0cu8,
		];

		// proof of alice is 0xd8b63c7168eef1bc3b00cdf73d1636429a26ab607b52f1de073b1f53edd9302d
		let alice_proof = vec![[
			0xd8u8, 0xb6u8, 0x3cu8, 0x71u8, 0x68u8, 0xeeu8, 0xf1u8, 0xbcu8, 0x3bu8, 0x00u8, 0xcdu8, 
			0xf7u8, 0x3du8, 0x16u8, 0x36u8, 0x42u8, 0x9au8, 0x26u8, 0xabu8, 0x60u8, 0x7bu8, 0x52u8, 
			0xf1u8, 0xdeu8, 0x07u8, 0x3bu8, 0x1fu8, 0x53u8, 0xedu8, 0xd9u8, 0x30u8, 0x2du8,
		]];

		// account id of bob is 0x5182a73e48bd6e814d0c2b41672d9cb8c87c4221b55bc08e0943198e90caad1f, claimed index 1  :01  00
		let bob_account: AccountId32 = AccountId32::from([
			0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
			0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
			0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
		]);

		// 0x84e8a1146740344fc59360423dcb97f58a6aa08e111b552c0a685af58c9bbb6d
		let bob_proof = vec![[
			0x84u8, 0xe8u8, 0xa1u8, 0x14u8, 0x67u8, 0x40u8, 0x34u8, 0x4fu8, 0xc5u8, 0x93u8, 0x60u8, 
			0x42u8, 0x3du8, 0xcbu8, 0x97u8, 0xf5u8, 0x8au8, 0x6au8, 0xa0u8, 0x8eu8, 0x11u8, 0x1bu8, 
			0x55u8, 0x2cu8, 0x0au8, 0x68u8, 0x5au8, 0xf5u8, 0x8cu8, 0x9bu8, 0xbbu8, 0x6du8,
		]];

		run_to_block(1);
		assert_eq!(System::block_number(), 1);

		let _ = Balances::deposit_creating(&alice_account, (CREATION_FEE + 10).into());

		// issue a claim class
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Claim(merkle_root),
		));

		assert_eq!(
			events(),
			[
				// System::Pallet<mock::Test>::Event::NewAccount(alice_account.clone(), 100),
			SystemEvent::NewAccount(alice_account.clone()).into(),	
			BalanceEvent::Endowed(alice_account.clone(), (CREATION_FEE + 10).into()).into(),

			SystemEvent::NewAccount(Pot::get()).into(),
			BalanceEvent::Endowed(Pot::get(), CREATION_FEE.into()).into(),
			BalanceEvent::Transfer(alice_account.clone(), Pot::get(), CREATION_FEE.into()).into(),
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(),0)),]
		);

		// alice claims with random proof
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, vec![[0u8; 32]],),
			NftError::UserNotInClaimList
		);

		// alice claims with alice's proof
		assert_ok!(Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof.clone(),));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::ClaimedToken(alice_account.clone(), 0)),]
		);

		// alice claims again
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof,),
			NftError::TokenAlreadyClaimed
		);

		let _ = Balances::deposit_creating(&bob_account, (CREATION_FEE + 10).into());
		// bob claim
		assert_ok!(Nft::claim(
			Origin::signed(bob_account.clone()),
			1,
			0,
			bob_proof.clone(),
		));

		events();

		
		// transfer some NFTs
		assert_ok!(Nft::transfer(
			Origin::signed(bob_account.clone()),
			alice_account.clone(),
			(0,1),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::TransferredToken(bob_account.clone(), alice_account.clone(), 0, 1)),]
		);

		// burn some NFTs
		assert_ok!(Nft::burn(
			Origin::signed(alice_account.clone()),
			(0,1),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::BurnedToken(alice_account.clone(), 0, 1)),]
		);
	})
}

#[test]
fn test_merge_type() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		let _ = Balances::deposit_creating(&account, (3 * CREATION_FEE + 10).into());

		run_to_block(1);
		assert_eq!(System::block_number(), 1);

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

		events();

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

		assert_eq!(
			events(),
			[
				Event::Nft(crate::Event::MintedToken(account.clone(), other_account.clone(), 0, 1)),
			  	Event::Nft(crate::Event::MintedToken(account.clone(), other_account.clone(), 1, 1)),
				  ]
		);

		// issue advanced NFTs
		assert_noop!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Merge(0, 999, false),
		),
		NftError::ClassIdNotFound
		);

		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Merge(0, 1, false),
		));

		events(); //clear the event

		// merge in wrong base order
		assert_noop!(Nft::merge(
			Origin::signed(other_account.clone()),
			2,
			(3, 0),
			(0, 0),
		),
		NftError::WrongMergeBase
		);

		// merge in right base order
		assert_ok!(Nft::merge(
			Origin::signed(other_account.clone()),
			2,
			(0, 0),
			(1, 0),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::MergedToken(other_account.clone(), 2)),]
		);


		// transfer merged NFTs
		assert_ok!(Nft::transfer(
			Origin::signed(other_account.clone()),
			account.clone(),
			(2,0),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::TransferredToken(other_account.clone(), account.clone(), 2, 0)),]
		);

		// burn some NFTs
		assert_ok!(Nft::burn(
			Origin::signed(account.clone()),
			(2,0),
		));

		assert_eq!(
			events(),
			[Event::Nft(crate::Event::BurnedToken(account.clone(), 2, 0)),]
		);

	})
}


#[test]
fn test_create_class_and_mint() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);
		let _ = Balances::deposit_creating(&account, (11 * CREATION_FEE + 10).into());
		
		run_to_block(1);
		assert_eq!(System::block_number(), 1);
		// create Transferable Unburnable class without start/end restrcition
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(enumflags2::BitFlags::from(ClassProperty::Transferable)),
			// Properties(ClassProperty::Transferable),
			None,
			None,
			ClassType::Simple(10),
		));

		//mint class Id=1 non-exist class 
		assert_noop!(Nft::mint(
			Origin::signed(other_account.clone()),
			other_account.clone(),
			1,
			CID::default(),
			1,
		),
		NftError::ClassIdNotFound);

		//mint 0 invalid instance quantity
		assert_noop!(Nft::mint(
			Origin::signed(other_account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			0,
		),
		NftError::InvalidQuantity);

		//mint 11 exceed the maximum instance limit
		assert_noop!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			11,
		),
		NftError::QuantityOverflow);

		//mint 5 instance with right ClassInfo owner
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			5,
		));

		//mint 5 instance with wrong ClassInfo owner
		assert_noop!(Nft::mint(
			Origin::signed(other_account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			5,
		),
		NftError::NoPermission);


		// create Transferable Unburnable class with start/end restrcition
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(enumflags2::BitFlags::from(ClassProperty::Transferable)),
			// Properties(ClassProperty::Transferable),
			10.into(),
			100.into(),
			ClassType::Simple(100),
		));

		run_to_block(2);
		assert_eq!(System::block_number(), 2);

		//mint 5 instance out of time
		assert_noop!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			1,
			CID::default(),
			5,
		),
		NftError::OutOfCampaignPeriod);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);
		//mint 5 instance within time
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			1,
			CID::default(),
			5,
		));

	});
}




#[test]
fn test_claim() {
	new_test_ext().execute_with(|| {
		// account id of Alice 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d ,claimed index 0: 00 00
		let alice_account: AccountId32 = AccountId32::from([
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		]);

		// root is 0x0c67caf461290cd463e535213f996e32736e65a2063783fde5036b71396dfb0c
		let merkle_root = [
			0x0cu8, 0x67u8, 0xcau8, 0xf4u8, 0x61u8, 0x29u8, 0x0cu8, 0xd4u8, 0x63u8, 0xe5u8, 0x35u8, 
			0x21u8, 0x3fu8, 0x99u8, 0x6eu8, 0x32u8, 0x73u8, 0x6eu8, 0x65u8, 0xa2u8, 0x06u8, 0x37u8, 
			0x83u8, 0xfdu8, 0xe5u8, 0x03u8, 0x6bu8, 0x71u8, 0x39u8, 0x6du8, 0xfbu8, 0x0cu8,
		];

		// proof of alice is 0xd8b63c7168eef1bc3b00cdf73d1636429a26ab607b52f1de073b1f53edd9302d
		let alice_proof = vec![[
			0xd8u8, 0xb6u8, 0x3cu8, 0x71u8, 0x68u8, 0xeeu8, 0xf1u8, 0xbcu8, 0x3bu8, 0x00u8, 0xcdu8, 
			0xf7u8, 0x3du8, 0x16u8, 0x36u8, 0x42u8, 0x9au8, 0x26u8, 0xabu8, 0x60u8, 0x7bu8, 0x52u8, 
			0xf1u8, 0xdeu8, 0x07u8, 0x3bu8, 0x1fu8, 0x53u8, 0xedu8, 0xd9u8, 0x30u8, 0x2du8,
		]];

		// account id of bob is 0x5182a73e48bd6e814d0c2b41672d9cb8c87c4221b55bc08e0943198e90caad1f, claimed index 1  :01  00
		let bob_account: AccountId32 = AccountId32::from([
			0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
			0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
			0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
		]);

		// 0x84e8a1146740344fc59360423dcb97f58a6aa08e111b552c0a685af58c9bbb6d
		let bob_proof = vec![[
			0x84u8, 0xe8u8, 0xa1u8, 0x14u8, 0x67u8, 0x40u8, 0x34u8, 0x4fu8, 0xc5u8, 0x93u8, 0x60u8, 
			0x42u8, 0x3du8, 0xcbu8, 0x97u8, 0xf5u8, 0x8au8, 0x6au8, 0xa0u8, 0x8eu8, 0x11u8, 0x1bu8, 
			0x55u8, 0x2cu8, 0x0au8, 0x68u8, 0x5au8, 0xf5u8, 0x8cu8, 0x9bu8, 0xbbu8, 0x6du8,
		]];

		run_to_block(1);
		assert_eq!(System::block_number(), 1);

		let _ = Balances::deposit_creating(&alice_account, (2 * CREATION_FEE + 10).into());

		// issue a claim class : class id, 0
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Claim(merkle_root),
		));

		// fake simple NFT : class id, 1
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Simple(100),
		));

		//-----------------------------------------//
		run_to_block(2);
		assert_eq!(System::block_number(), 2);

		// alice claim out of time
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof.clone(),),
			NftError::OutOfCampaignPeriod
		);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);

		// Claim non-existed type
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 1, alice_proof.clone(),),
			NftError::ClassClaimedListNotFound
			//WrongClassType // should we raise this error first??? This can never be triggered.
		);

		// alice claims with alice's proof
		assert_ok!(Nft::claim(
			Origin::signed(alice_account.clone()),
			0,
			0,
			alice_proof.clone(),
		));

	});
}

#[test]
// merge base type test
fn test_merge() {
	new_test_ext().execute_with(|| {
		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let other_account: AccountId32 = AccountId32::from([1u8; 32]);

		let _ = Balances::deposit_creating(&account, (6 * CREATION_FEE + 10).into());

		run_to_block(1);
		assert_eq!(System::block_number(), 1);

		// issue basic unburnable NFTs : class id 0, 1
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(10),
		));

		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable.into()),
			None,
			None,
			ClassType::Simple(10),
		));


		// issue basic burnable NFTs : class id 2, 3
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(10),
		));

		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable| ClassProperty::Burnable),
			None,
			None,
			ClassType::Simple(10),
		));

		events();

		// mint unburnable NFTs  : class id 0, 1
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			0,
			CID::default(),
			10,
		));

		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			1,
			CID::default(),
			10,
		));

		assert_eq!(
			events(),
			[
				Event::Nft(crate::Event::MintedToken(account.clone(), other_account.clone(), 0, 10)),
			  	Event::Nft(crate::Event::MintedToken(account.clone(), other_account.clone(), 1, 10)),
				  ]
		);

		// mint burnable NFTs  : class id 2, 3
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			2,
			CID::default(),
			10,
		));
		assert_ok!(Nft::mint(
			Origin::signed(account.clone()),
			other_account.clone(),
			3,
			CID::default(),
			10,
		));


		// issue advanced NFTs
		assert_noop!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Merge(0, 1, true),
		),
		NftError::NonBurnable);

		// issue base unburn merge NFTs : class id 4
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Merge(2, 3, false),
		));


		// issue base burn merge NFTs: class id 5
		assert_ok!(Nft::create_class(
			Origin::signed(account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			10.into(),
			100.into(),
			ClassType::Merge(2, 3, true),
		));

		events(); //clear the event

		//---------------------------------------------//
		//---unburn merge NFT--------------------------//
		// merge out of time
		assert_noop!(Nft::merge(
			Origin::signed(other_account.clone()),
			4,
			(2, 9),
			(3, 9),
		),
		NftError::OutOfCampaignPeriod);

		run_to_block(11);
		assert_eq!(System::block_number(), 11);

		// merge existed class but non-existed token 
		assert_noop!(Nft::merge(
			Origin::signed(other_account.clone()),
			4,
			(2, 11),
			(3, 11),
		),
		NftError::TokenNotFound);

		// merge existed class and existed token
		assert_ok!(Nft::merge(
			Origin::signed(other_account.clone()),
			4,
			(2, 9),
			(3, 9),
		));
		assert_eq!(
			events(),
			[Event::Nft(crate::Event::MergedToken(other_account.clone(), 4)),]
		);

		// merge existed class and existed token again for used token
		assert_noop!(Nft::merge(
			Origin::signed(other_account.clone()),
			4,
			(2, 9),
			(3, 9),
		),
		NftError::TokenUsed);

		//---------------------//
		//---burn merge NFT--------------------------//
		assert_ok!(Nft::merge(
			Origin::signed(other_account.clone()),
			5,
			(2, 9),
			(3, 9),
		));
		assert_eq!(
			events(),
			[Event::Nft(crate::Event::MergedToken(other_account.clone(), 5)),]
		);
		let random_account: AccountId32 = AccountId32::from([
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00,
		]);
		assert_eq!(Nft::owner((2,9)).unwrap_or(random_account.clone()), random_account);

	});
}