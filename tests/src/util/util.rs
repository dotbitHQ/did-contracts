use super::{super::ckb_types_relay::*, constants::*, error};
use chrono::{DateTime, NaiveDateTime, Utc};
use ckb_hash::{blake2b_256, Blake2bBuilder};
use ckb_types::{bytes, packed::*, prelude::*};
use lazy_static::lazy_static;
use serde_json::Value;
use std::{
    env,
    error::Error,
    fs::File,
    io,
    io::{BufRead, BufReader, Lines},
    path::PathBuf,
};

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
    let mut blake2b = Blake2bBuilder::new(32).personal(b"sparsemerkletree").key(&[]).build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
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
