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

#[test]
fn test_insert_fix_data() {
    new_test_ext().execute_with(|| {

        run_to_block(1);

		// account id of Alice 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		let account: AccountId32 = AccountId32::from([
			0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
			0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
			0xa5, 0x6d, 0xa2, 0x7d,
		]);

		let block_number: u32 = 10000;
		let layer_one_blocknumber: u32 = 10;

		let index = 0;
		let eth_address_str = "4d88dc5d528a33e4b8be579e9476715f60060582";
		let decoded_address = hex::decode(eth_address_str).unwrap();
		let mut eth_address = [0_u8; 20];
		eth_address[0..20].copy_from_slice(&decoded_address[0..20]);
		let signature_str = "318400f0f9bd15f0d8842870b510e996dffc944b77111ded03a4255c66e82d427132e765d5e6bb21ba046dbb98e28bb28cb2bebe0c8aced2c547aca60a5548921c";
		let decoded_signature = hex::decode(signature_str).unwrap();
		let mut signature = [0_u8; 65];
		signature[0..65].copy_from_slice(&decoded_signature[0..65]);

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
