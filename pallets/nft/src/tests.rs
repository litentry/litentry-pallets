use super::*;
use crate::mock::{Event, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;

// -> (AccountId32, AccountId32)


fn initial_account() -> (AccountId32, AccountId32) {
	let alice_account: AccountId32 = AccountId32::from([
		0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9, 0x9f,
		0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d,
		0xa2, 0x7d,
	]);

	let bob_account: AccountId32 = AccountId32::from([
		0x51u8, 0x82u8, 0xa7u8, 0x3eu8, 0x48u8, 0xbdu8, 0x6eu8, 0x81u8, 0x4du8, 0x0cu8, 0x2bu8,
		0x41u8, 0x67u8, 0x2du8, 0x9cu8, 0xb8u8, 0xc8u8, 0x7cu8, 0x42u8, 0x21u8, 0xb5u8, 0x5bu8,
		0xc0u8, 0x8eu8, 0x09u8, 0x43u8, 0x19u8, 0x8eu8, 0x90u8, 0xcau8, 0xadu8, 0x1fu8,
	]);

	let start_wealth: u64 = (5 * CREATION_FEE + 10).into();
	let _ = Balances::deposit_creating(&alice_account, start_wealth);
	let _ = Balances::deposit_creating(&bob_account, start_wealth);

	run_to_block(1);

	let _ = Balances::deposit_creating(&Pot::get(), 1);

	return (alice_account.clone(), bob_account.clone());
}

// ClassIdNotFound,
// 		

// 		
// 		/// Token not found
// 		TokenNotFound,
// 		/// Wrong class type
// 		WrongClassType,
// 		/// Merge nft's base nfts are not provided correctly
// 		WrongMergeBase,
// 		/// Use already used token to merge new token
// 		TokenUsed,
// 		

// 		/// user cannot pay NFT class creation fee
// 		CreationFeeNotPaid,


// 	/// Merged NFT token. \[owner, class_id\]
// 	MergedToken(T::AccountId, ClassIdOf<T>),
// 	/// Transferred NFT token. \[from, to, class_id, token_id\]
// 	TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
// 	/// Burned NFT token. \[owner, class_id, token_id\]
// 	BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>),
// }


#[test]
// Test for general Event:
// CreatedClass(T::AccountId, ClassIdOf<T>) 
// Test for Error:
fn test_general_event() {
	new_test_ext().execute_with(|| {
		let (alice_account, _bob_account) = initial_account();

		let merkle_root = [
			0x0cu8, 0x67u8, 0xcau8, 0xf4u8, 0x61u8, 0x29u8, 0x0cu8, 0xd4u8, 0x63u8, 0xe5u8, 0x35u8,
			0x21u8, 0x3fu8, 0x99u8, 0x6eu8, 0x32u8, 0x73u8, 0x6eu8, 0x65u8, 0xa2u8, 0x06u8, 0x37u8,
			0x83u8, 0xfdu8, 0xe5u8, 0x03u8, 0x6bu8, 0x71u8, 0x39u8, 0x6du8, 0xfbu8, 0x0cu8,
		];

		// Simple: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties::default(),
			None,
			None,
			ClassType::Simple(100),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[0],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 0)),
		);

		// Claim: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Claim(merkle_root),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[1],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 1)),
		);

		// Merge: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Merge(0, 1, false),
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[2],
			Event::Nft(crate::Event::CreatedClass(alice_account.clone(), 2)),
		);

	})
}


#[test]
// Test for Simple type Event:
// MintedToken(T::AccountId, T::AccountId, ClassIdOf<T>, u32)
// TransferredToken(T::AccountId, T::AccountId, ClassIdOf<T>, TokenIdOf<T>)
// BurnedToken(T::AccountId, ClassIdOf<T>, TokenIdOf<T>)

// Test for Error:
// OutOfCampaignPeriod,
// NonTransferable,
// NonBurnable,
// NoPermission,
// InvalidQuantity,
// QuantityOverflow

fn test_minted_token_event() {
	new_test_ext().execute_with(|| {
		let (alice_account, bob_account) = initial_account();

		// Simple: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Burnable.into()),
			None,
			None,
			ClassType::Simple(100),
		));

		// mint some NFTs
		assert_ok!(Nft::mint(
			Origin::signed(alice_account.clone()),
			bob_account.clone(),
			0,
			CID::default(),
			2,
		));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[1],
			Event::Nft(crate::Event::MintedToken(alice_account.clone(), bob_account.clone(), 0, 2)),
		);
	})
}


#[test]
// Test for Claim type Event:
// ClaimedToken(T::AccountId, ClassIdOf<T>)
// Test for Error:
// UserNotInClaimList
// TokenAlreadyClaimed
fn test_claimed_token_event() {
	new_test_ext().execute_with(|| {
		let (alice_account, _bob_account) = initial_account();

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

		// 0x84e8a1146740344fc59360423dcb97f58a6aa08e111b552c0a685af58c9bbb6d
		let _bob_proof = vec![[
			0x84u8, 0xe8u8, 0xa1u8, 0x14u8, 0x67u8, 0x40u8, 0x34u8, 0x4fu8, 0xc5u8, 0x93u8, 0x60u8,
			0x42u8, 0x3du8, 0xcbu8, 0x97u8, 0xf5u8, 0x8au8, 0x6au8, 0xa0u8, 0x8eu8, 0x11u8, 0x1bu8,
			0x55u8, 0x2cu8, 0x0au8, 0x68u8, 0x5au8, 0xf5u8, 0x8cu8, 0x9bu8, 0xbbu8, 0x6du8,
		]];

		// Claim: CreatedClass Event
		assert_ok!(Nft::create_class(
			Origin::signed(alice_account.clone()),
			CID::default(),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			None,
			None,
			ClassType::Claim(merkle_root),
		));

		// alice claims with random proof
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, vec![[0u8; 32]],),
			NftError::UserNotInClaimList
		);

		// alice claims with alice's proof
		assert_ok!(Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof.clone(),));
		assert_eq!(
			events_filter::<crate::Event::<Test>>()[1],
			Event::Nft(crate::Event::ClaimedToken(alice_account.clone(), 0))
		);

		// alice claims again
		assert_noop!(
			Nft::claim(Origin::signed(alice_account.clone()), 0, 0, alice_proof,),
			NftError::TokenAlreadyClaimed
		);
	})
}











