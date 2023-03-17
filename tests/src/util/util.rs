use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;
use std::{env, io, str};

use chrono::{DateTime, NaiveDateTime, Utc};
use ckb_hash::{blake2b_256, Blake2bBuilder};
use ckb_types::bytes;
use ckb_types::packed::*;
use ckb_types::prelude::*;
use lazy_static::lazy_static;
use serde_json::Value;
use sparse_merkle_tree::H256;

use super::super::ckb_types_relay::*;
use super::constants::*;
use super::error;

lazy_static! {
    pub static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

pub fn contains_error(message: &str, err_code: error::ErrorCode) -> bool {
    let err_str = format!("ValidationFailure({})", (err_code as i8).to_string());
    message.contains(&err_str)
}

pub fn hex_to_bytes(input: &str) -> Vec<u8> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect("Expect input to valid hex")
    }
}

pub fn hex_to_bytes_2(input: &str) -> bytes::Bytes {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        bytes::Bytes::new()
    } else {
        let data: Vec<u8> = hex::decode(hex).expect("Expect input to valid hex");
        bytes::Bytes::from(data)
    }
}

pub fn bytes_to_hex(input: &[u8]) -> String {
    if input.is_empty() {
        String::from("0x")
    } else {
        String::from("0x") + &hex::encode(input)
    }
}

pub fn hex_to_byte32(input: &str) -> Result<Byte32, Box<dyn Error>> {
    let bytes = hex_to_bytes(input);

    Ok(byte32_new(&bytes))
}

pub fn hex_to_u64(input: &str) -> Result<u64, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Ok(0u64)
    } else {
        Ok(u64::from_str_radix(hex, 16)?)
    }
}

pub fn merge_json(target: &mut Value, source: Value) {
    if source.is_null() {
        return;
    }

    match (target, source) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_json(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a @ &mut Value::Array(_), Value::Array(b)) => {
            let a = a.as_array_mut().unwrap();
            for v in b {
                a.push(v);
            }
        }
        (a, b) => *a = b,
    }
}

pub fn blake2b_smt<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = Blake2bBuilder::new(32).personal(b"ckb-default-hash").key(&[]).build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn gen_smt_key_from_account(account: &str) -> [u8; 32] {
    let account_id = account_to_id(account);
    let mut key = [0u8; 32];
    let key_pre = [account_id, vec![0u8; 12]].concat();
    key.copy_from_slice(&key_pre);
    key
}

pub fn gen_smt_value_for_reverse_record_smt(nonce: u32, account: &[u8]) -> H256 {
    let raw = [nonce.to_le_bytes().to_vec(), account.to_vec()].concat();
    blake2b_256(raw).into()
}

pub fn get_type_id_bytes(name: &str) -> Vec<u8> {
    hex_to_bytes(
        TYPE_ID_TABLE
            .get(name)
            .expect(&format!("Can not find type ID for {}", name)),
    )
}

pub fn account_to_id(account: &str) -> Vec<u8> {
    let hash = blake2b_256(account);
    hash.get(..ACCOUNT_ID_LENGTH).unwrap().to_vec()
}

pub fn account_to_id_hex(account: &str) -> String {
    format!("0x{}", hex_string(account_to_id(account).as_slice()))
}

pub fn prepend_molecule_like_length(raw: Vec<u8>) -> Vec<u8> {
    // Prepend length of bytes to raw data, include the bytes of length itself.
    let mut entity = (raw.len() as u32 + 4).to_le_bytes().to_vec();
    entity.extend(raw);

    entity
}

pub fn read_lines(file_name: &str) -> io::Result<Lines<BufReader<File>>> {
    let dir = env::current_dir().unwrap();
    let mut file_path = PathBuf::new();
    file_path.push(dir);
    file_path.push("data");
    file_path.push(file_name);

    // Read record keys from file, then sort them.
    let file = File::open(file_path)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn gen_timestamp(datetime: &str) -> u64 {
    let navie_datetime =
        NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S").expect("Invalid datetime format.");
    let datetime = DateTime::<Utc>::from_utc(navie_datetime, Utc);
    datetime.timestamp() as u64
}

pub fn gen_register_fee(account_length: usize, has_inviter: bool) -> u64 {
    let price_in_usd = match account_length {
        1 => ACCOUNT_PRICE_1_CHAR,
        2 => ACCOUNT_PRICE_2_CHAR,
        3 => ACCOUNT_PRICE_3_CHAR,
        4 => ACCOUNT_PRICE_4_CHAR,
        _ => ACCOUNT_PRICE_5_CHAR,
    };

    let price_in_ckb = price_in_usd / CKB_QUOTE * 100_000_000;

    if has_inviter {
        price_in_ckb * (RATE_BASE - INVITED_DISCOUNT) / RATE_BASE
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account_length as u64 + 4) * 100_000_000
    } else {
        price_in_ckb
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account_length as u64 + 4) * 100_000_000
    }
}

pub fn gen_register_fee_v2(account: &str, account_length: usize, has_inviter: bool) -> u64 {
    let price_in_usd = match account_length {
        1 => ACCOUNT_PRICE_1_CHAR,
        2 => ACCOUNT_PRICE_2_CHAR,
        3 => ACCOUNT_PRICE_3_CHAR,
        4 => ACCOUNT_PRICE_4_CHAR,
        _ => ACCOUNT_PRICE_5_CHAR,
    };

    let price_in_ckb = price_in_usd / CKB_QUOTE * 100_000_000;

    if has_inviter {
        price_in_ckb * (RATE_BASE - INVITED_DISCOUNT) / RATE_BASE
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account.as_bytes().len() as u64) * 100_000_000
    } else {
        price_in_ckb
            + ACCOUNT_BASIC_CAPACITY
            + ACCOUNT_PREPARED_FEE_CAPACITY
            + (account.as_bytes().len() as u64) * 100_000_000
    }
}

pub fn gen_account_cell_capacity(length: u64) -> u64 {
    ((length + 4) * 100_000_000) + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY
}

/// Parse u64 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u64(field_name: &str, field: &Value, default: Option<u64>) -> u64 {
    if let Some(val) = field.as_u64() {
        val
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u64 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u32 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u32(field_name: &str, field: &Value, default: Option<u32>) -> u32 {
    if let Some(val) = field.as_u64() {
        val as u32
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u32 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u8 in JSON
pub fn parse_json_u8(field_name: &str, field: &Value, default: Option<u8>) -> u8 {
    if let Some(val) = field.as_u64() {
        if val > u8::MAX as u64 {
            panic!("{} should be u8", field_name)
        } else {
            val as u8
        }
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u8 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse hex string in JSON
///
/// Prefix "0x" is optional.
pub fn parse_json_hex(field_name: &str, field: &Value) -> Vec<u8> {
    let mut hex = field.as_str().expect(&format!("{} is missing", field_name));
    hex = hex.trim_start_matches("0x");

    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect(&format!("{} is should be hex string", field_name))
    }
}

/// Parse hex string in JSON, if it is not exist return the default value.
pub fn parse_json_hex_with_default(field_name: &str, field: &Value, default: Vec<u8>) -> Vec<u8> {
    if field.is_null() {
        default
    } else {
        parse_json_hex(field_name, field)
    }
}
