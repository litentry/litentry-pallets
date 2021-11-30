use crate::mock::*;

use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use parity_crypto::publickey::{sign, Generator, KeyPair, Message, Random};
use parity_crypto::Keccak256;
use sp_runtime::AccountId32;

fn generate_msg(account: &AccountId32, block_number: u32) -> Message {
    let mut bytes = b"\x19Ethereum Signed Message:\n51Link Litentry: ".encode();
    let mut account_vec = account.encode();
    let mut expiring_block_number_vec = block_number.encode();

    bytes.append(&mut account_vec);
    bytes.append(&mut expiring_block_number_vec);

    Message::from(bytes.keccak256())
}

fn generate_sig(key_pair: &KeyPair, msg: &Message) -> [u8; 65] {
    sign(key_pair.secret(), &msg).unwrap().into_electrum()
}

#[test]
fn test_expired_block_number_eth() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::from([0u8; 32]);
        let block_number: u32 = 100;
        let layer_one_blocknumber: u32 = 1000;

        let mut gen = Random {};
        let key_pair = gen.generate();

        let msg = generate_msg(&account, block_number);
        let sig = generate_sig(&key_pair, &msg);

        assert_noop!(
            SgxAccountLinker::do_link_eth(
                account.clone(),
                0,
                key_pair.address().to_fixed_bytes(),
                block_number,
                layer_one_blocknumber,
                sig
            ),
            SgxAccountLinkerError::LinkRequestExpired
        );
    });
}

#[test]
fn test_invalid_expiring_block_number_eth() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::from([0u8; 32]);
        let block_number: u32 = crate::EXPIRING_BLOCK_NUMBER_MAX + 10;
        let layer_one_blocknumber: u32 = 1;

        let mut gen = Random {};
        let key_pair = gen.generate();

        let msg = generate_msg(&account, block_number);
        let sig = generate_sig(&key_pair, &msg);

        assert_noop!(
            SgxAccountLinker::do_link_eth(
                account.clone(),
                0,
                key_pair.address().to_fixed_bytes(),
                block_number,
                layer_one_blocknumber,
                sig
            ),
            SgxAccountLinkerError::InvalidExpiringBlockNumber
        );
    });
}

#[test]
fn test_unexpected_address_eth() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::from([72u8; 32]);
        let block_number: u32 = 99999;
        let layer_one_blocknumber: u32 = 10;

        let mut gen = Random {};
        let key_pair = gen.generate();

        let msg = generate_msg(&account, block_number);
        let sig = generate_sig(&key_pair, &msg);

        assert_noop!(
            SgxAccountLinker::do_link_eth(
                account.clone(),
                0,
                gen.generate().address().to_fixed_bytes(),
                block_number,
                layer_one_blocknumber,
                sig
            ),
            SgxAccountLinkerError::UnexpectedAddress
        );
    });
}

#[test]
fn test_insert_eth_address() {
    new_test_ext().execute_with(|| {
        run_to_block(1);

        let account: AccountId32 = AccountId32::from([5u8; 32]);
        let block_number: u32 = 99999;
        let layer_one_blocknumber: u32 = 10;

        let mut gen = Random {};
        let mut expected_vec = Vec::new();

        for i in 0..(MAX_ETH_LINKS) {
            let key_pair = gen.generate();

            let msg = generate_msg(&account, block_number + i as u32);
            let sig = generate_sig(&key_pair, &msg);

            assert_ok!(SgxAccountLinker::do_link_eth(
                account.clone(),
                i as u32,
                key_pair.address().to_fixed_bytes(),
                block_number + i as u32,
                layer_one_blocknumber,
                sig
            ));

            assert_eq!(SgxAccountLinker::eth_addresses(&account).len(), i + 1);
            expected_vec.push(key_pair.address().to_fixed_bytes());
            assert_eq!(
                events(),
                [Event::SgxAccountLinker(crate::Event::EthAddressLinked(
                    account.clone(),
                    key_pair.address().to_fixed_bytes().to_vec()
                )),]
            );
        }
        assert_eq!(SgxAccountLinker::eth_addresses(&account), expected_vec);
    });
}

#[test]
fn test_update_eth_address() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::from([40u8; 32]);
        let block_number: u32 = 99999;
        let layer_one_blocknumber: u32 = 10;

        let mut gen = Random {};
        for i in 0..(MAX_ETH_LINKS) {
            let key_pair = gen.generate();
            let msg = generate_msg(&account, block_number + i as u32);
            let sig = generate_sig(&key_pair, &msg);

            assert_ok!(SgxAccountLinker::do_link_eth(
                account.clone(),
                i as u32,
                key_pair.address().to_fixed_bytes(),
                block_number + i as u32,
                layer_one_blocknumber,
                sig
            ));
        }

        let index: u32 = 2 as u32;
        // Retrieve previous addr
        let addr_before_update = SgxAccountLinker::eth_addresses(&account)[index as usize];
        // Update addr at slot `index`
        let key_pair = gen.generate();
        let block_number = block_number + 9 as u32;
        let msg = generate_msg(&account, block_number);
        let sig = generate_sig(&key_pair, &msg);

        assert_ok!(SgxAccountLinker::do_link_eth(
            account.clone(),
            index,
            key_pair.address().to_fixed_bytes(),
            block_number,
            layer_one_blocknumber,
            sig
        ));

        let updated_addr = SgxAccountLinker::eth_addresses(&account)[index as usize];
        assert_ne!(updated_addr, addr_before_update);
        assert_eq!(updated_addr, key_pair.address().to_fixed_bytes());
    });
}

#[test]
fn test_eth_address_pool_overflow() {
    new_test_ext().execute_with(|| {
        let account: AccountId32 = AccountId32::from([113u8; 32]);
        let block_number: u32 = 99999;
        let layer_one_blocknumber: u32 = 10;

        let mut gen = Random {};
        let mut expected_vec = Vec::new();

        for index in 0..(MAX_ETH_LINKS * 2) {
            let key_pair = gen.generate();

            let msg = generate_msg(&account, block_number);
            let sig = generate_sig(&key_pair, &msg);

            assert_ok!(SgxAccountLinker::do_link_eth(
                account.clone(),
                index as u32,
                key_pair.address().to_fixed_bytes(),
                block_number,
                layer_one_blocknumber,
                sig
            ));

            if index < MAX_ETH_LINKS {
                expected_vec.push(key_pair.address().to_fixed_bytes());
            } else {
                expected_vec[MAX_ETH_LINKS - 1] = key_pair.address().to_fixed_bytes();
            }
        }
        assert_eq!(
            SgxAccountLinker::eth_addresses(&account).len(),
            MAX_ETH_LINKS
        );
        assert_eq!(SgxAccountLinker::eth_addresses(&account), expected_vec);
    });
}

use sp_core::{ed25519 as Ed25519, Pair};
#[test]
fn test_insert_fix_data() {
    new_test_ext().execute_with(|| {
        run_to_block(1);

        const TEST_SEED: [u8; 32] = *b"12345678901234567890123456789012";
        let test_account = Ed25519::Pair::from_seed(&TEST_SEED);
        let account: AccountId32 = test_account.public().into();
        let block_number: u32 = 10000;
        let layer_one_blocknumber: u32 = 10;

        let index = 0;
        let mut eth_address: [u8; 20] = [
            134, 249, 11, 141, 35, 15, 14, 35, 186, 87, 252, 83, 209, 126, 64, 176, 240, 183, 253,
            28,
        ];
        let mut signature: [u8; 65] = [
            99, 207, 133, 198, 25, 165, 160, 210, 73, 99, 151, 44, 62, 197, 18, 249, 46, 167, 34,
            134, 105, 226, 109, 209, 201, 43, 254, 191, 86, 187, 5, 184, 7, 229, 171, 144, 226,
            216, 209, 125, 85, 85, 236, 172, 167, 190, 172, 203, 67, 102, 207, 104, 5, 75, 46, 252,
            134, 75, 224, 82, 64, 11, 41, 58, 27,
        ];

        println!("account: {:?}", account.encode());
        println!("addr_expected: {:?}", eth_address);
        println!("exp number: {:?}", block_number);
        println!("sign: {:?}", signature);

        assert_ok!(SgxAccountLinker::do_link_eth(
            account.clone(),
            index,
            eth_address,
            block_number,
            layer_one_blocknumber,
            signature
        ));
    });
}
