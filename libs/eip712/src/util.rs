use std::prelude::v1::*;

use bech32::{ToBase32, Variant};
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

pub fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak::v256();
    hasher.update(data);

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    output.to_vec()
}

pub fn parse_type(type_: &str) -> &str {
    if type_.ends_with("[]") {
        &type_[0..(type_.len() - 2)]
    } else {
        type_
    }
}

pub fn address_prefix() -> String {
    if env!("NETWORK") == "mainnet" {
        String::from("ckb")
    } else {
        String::from("ckt")
    }
}

pub fn to_short_address(code_hash_index: Vec<u8>, args: Vec<u8>) -> Result<String, bech32::Error> {
    // This is the payload of legacy address.
    let data = [vec![1], code_hash_index, args].concat();

    bech32::encode(&address_prefix(), data.to_base32(), Variant::Bech32)
}

pub fn to_full_address(code_hash: Vec<u8>, hash_type: Vec<u8>, args: Vec<u8>) -> Result<String, bech32::Error> {
    // This is the payload of full address.
    let data = [vec![0u8], code_hash, hash_type, args].concat();

    bech32::encode(&address_prefix(), data.to_base32(), Variant::Bech32m)
}

const TRON_ADDR_PREFIX: u8 = 0x41;

pub fn to_tron_address(pubkey_hash: impl AsRef<[u8]>) -> String {
    let mut payload = vec![TRON_ADDR_PREFIX];
    payload.extend(pubkey_hash.as_ref());
    b58encode_check(payload)
}

const DOGE_ADDR_PREFIX: u8 = 0x1E;

pub fn to_doge_address(pubkey_hash: impl AsRef<[u8]>) -> String {
    let mut payload = vec![DOGE_ADDR_PREFIX];
    payload.extend(pubkey_hash.as_ref());
    b58encode_check(payload)
}

fn b58encode_check<T: AsRef<[u8]>>(raw: T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_ref());
    let digest1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&digest1);
    let digest = hasher.finalize();

    let mut input = raw.as_ref().to_owned();
    input.extend(&digest[..4]);
    let mut output = String::new();
    bs58::encode(&input).into(&mut output).unwrap();

    output
}

pub fn to_semantic_capacity(capacity: u64) -> String {
    let capacity_str = capacity.to_string();
    let length = capacity_str.len();
    let mut ret = String::new();
    if length > 8 {
        let integer = &capacity_str[0..length - 8];
        let mut decimal = &capacity_str[length - 8..length];
        decimal = decimal.trim_end_matches("0");
        if decimal.is_empty() {
            ret = ret + integer + " CKB";
        } else {
            ret = ret + integer + "." + decimal + " CKB";
        }
    } else {
        if capacity_str == "0" {
            ret = String::from("0 CKB");
        } else {
            let padded_str = format!("{:0>8}", capacity_str);
            let decimal = padded_str.trim_end_matches("0");
            ret = ret + "0." + decimal + " CKB";
        }
    }

    ret
}

pub fn to_semantic_currency(value: u64, unit: &str) -> String {
    let precision = 6;
    let capacity_str = value.to_string();
    let length = capacity_str.len();
    let mut ret = String::new();
    if length > precision {
        let integer = &capacity_str[0..length - precision];
        let mut decimal = &capacity_str[length - precision..length];
        decimal = decimal.trim_end_matches("0");
        if decimal.is_empty() {
            ret = ret + integer + " " + unit;
        } else {
            ret = ret + integer + "." + decimal + " " + unit;
        }
    } else {
        if capacity_str == "0" {
            ret = format!("0 {}", unit);
        } else {
            let padded_str = format!("{:0>6}", capacity_str);
            let decimal = padded_str.trim_end_matches("0");
            ret = ret + "0." + decimal + " " + unit;
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_eip712_to_short_address() {
        // Copy from https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md
        let code_hash_index = vec![0];
        let args = hex::decode("b39bbc0b3673c7d36450bc14cfcdad2d559c6c64").unwrap();

        let expected = "ckt1qyqt8xaupvm8837nv3gtc9x0ekkj64vud3jq5t63cs";
        let address = to_short_address(code_hash_index, args).unwrap();
        assert_eq!(&address, expected);
    }

    #[test]
    fn test_eip712_to_full_address() {
        // Copy from https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md
        let code_hash = hex::decode("9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8").unwrap();
        let hash_type = vec![1];
        let args = hex::decode("b39bbc0b3673c7d36450bc14cfcdad2d559c6c64").unwrap();

        let expected =
            "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqdnnw7qkdnnclfkg59uzn8umtfd2kwxceqgutnjd";
        let address = to_full_address(code_hash, hash_type, args).unwrap();
        assert_eq!(&address, expected);
    }

    #[test]
    fn test_eip712_to_tron_address() {
        let payload = hex::decode("8840E6C55B9ADA326D211D818C34A994AECED808").unwrap();

        let expected = "TNPeeaaFB7K9cmo4uQpcU32zGK8G1NYqeL";
        let address = to_tron_address(payload);

        assert_eq!(&address, expected);
    }

    #[test]
    fn test_eip712_to_doge_address() {
        let payload = hex::decode("faeb6478ecfdb01e00409130fd51c46e8e04ef01").unwrap();

        let expected = "DU1qTa77uRizv4JGR8Ydj6Yrs73GVT2pFR";
        let address = to_doge_address(payload);

        assert_eq!(&address, expected);
    }

    #[test]
    fn test_eip712_to_semantic_capacity() {
        let expected = "0 CKB";
        let result = to_semantic_capacity(0);
        assert_eq!(result, expected);

        let expected = "1 CKB";
        let result = to_semantic_capacity(100_000_000);
        assert_eq!(result, expected);

        let expected = "0.0001 CKB";
        let result = to_semantic_capacity(10_000);
        assert_eq!(result, expected);

        let expected = "1000.0001 CKB";
        let result = to_semantic_capacity(100_000_010_000);
        assert_eq!(result, expected);

        let expected = "1000 CKB";
        let result = to_semantic_capacity(100_000_000_000);
        assert_eq!(result, expected);
    }
}
