use std::prelude::v1::*;

use bech32::{ToBase32, Variant};
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

#[cfg(feature = "mainnet")]
const HRP: &str = "ckb";
#[cfg(not(feature = "mainnet"))]
const HRP: &str = "ckt";

pub fn to_short_address(code_hash_index: Vec<u8>, args: Vec<u8>) -> Result<String, bech32::Error> {
    // This is the payload of legacy address.
    let data = [vec![1], code_hash_index, args].concat();

    bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32)
}

pub fn to_full_address(code_hash: Vec<u8>, hash_type: Vec<u8>, args: Vec<u8>) -> Result<String, bech32::Error> {
    // This is the payload of full address.
    let data = [vec![0u8], code_hash, hash_type, args].concat();

    bech32::encode(&HRP.to_string(), data.to_base32(), Variant::Bech32m)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_eip712_to_semantic_address() {
        // Copy from https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md
        let code_hash = vec![
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200, 142, 93, 75, 101,
            168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ];
        let hash_type = vec![1];
        // b39bbc0b3673c7d36450bc14cfcdad2d559c6c64
        let args = vec![
            179, 155, 188, 11, 54, 115, 199, 211, 100, 80, 188, 20, 207, 205, 173, 45, 85, 156, 108, 100,
        ];

        let expected =
            "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqdnnw7qkdnnclfkg59uzn8umtfd2kwxceqgutnjd";
        let address = to_full_address(code_hash, hash_type, args).unwrap();
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
