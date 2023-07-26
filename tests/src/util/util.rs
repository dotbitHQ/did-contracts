use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;
use std::{env, io, str};

use chrono::{DateTime, NaiveDateTime, Utc};
use ckb_hash::{blake2b_256, Blake2bBuilder};
use ckb_types::prelude::hex_string;
use ckb_types::{bytes, packed as ckb_packed};
use das_types_std::constants::*;
use das_types_std::packed::*;
use das_types_std::prelude::*;
use lazy_static::lazy_static;
use serde_json::{json, Value};
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

pub fn hex_to_byte32(input: &str) -> Result<ckb_packed::Byte32, Box<dyn Error>> {
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

pub fn usd_to_ckb(usd: u64) -> u64 {
    usd / CKB_QUOTE * ONE_CKB
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

    let price_in_ckb = price_in_usd / CKB_QUOTE * ONE_CKB;

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

fn gen_account_char(char: &str, char_set_type: CharSetType) -> AccountChar {
    AccountChar::new_builder()
        .char_set_name(Uint32::from(char_set_type as u32))
        .bytes(Bytes::from(char.as_bytes()))
        .build()
}

pub fn gen_account_chars(chars: Vec<impl AsRef<str>>) -> AccountChars {
    let mut builder = AccountChars::new_builder();
    for char in chars {
        let char = char.as_ref();
        // Filter empty chars come from str.split("").
        if char.is_empty() {
            continue;
        }

        // ⚠️ For testing only, the judgement is not accurate, DO NOT support multiple emoji with more than 4 bytes.
        if char.len() != 1 {
            if RE_ZH_CHAR.is_match(char) {
                builder = builder.push(gen_account_char(char, CharSetType::ZhHans))
            } else {
                builder = builder.push(gen_account_char(char, CharSetType::Emoji))
            }
        } else {
            let raw_char = char.chars().next().unwrap();
            if raw_char.is_digit(10) {
                builder = builder.push(gen_account_char(char, CharSetType::Digit))
            } else {
                builder = builder.push(gen_account_char(char, CharSetType::En))
            }
        }
    }

    builder.build()
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

/// Parse string in JSON
///
/// All string will be treated as utf8 encoding.
pub fn parse_json_str<'a>(field_name: &str, field: &'a Value) -> &'a str {
    field.as_str().expect(&format!("{} is missing", field_name))
}

pub fn parse_json_str_with_default<'a>(field_name: &str, field: &'a Value, default: &'a str) -> &'a str {
    if field.is_null() {
        default
    } else {
        parse_json_str(field_name, field)
    }
}

/// Parse array in JSON
pub fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a [Value] {
    field
        .as_array()
        .map(|v| v.as_slice())
        .expect(&format!("{} is missing", field_name))
}

/// Parse struct Script to hex of molecule encoding, if field is null will return Script::default()
///
/// Example:
/// ```json
/// {
///     code_hash: "{{xxx-cell-type}}"
///     hash_type: "type", // could be omit if it is "type"
///     args: "" // could be omit if it it empty
/// }
/// ```
pub fn parse_json_script_to_mol(field_name: &str, field: &Value) -> Script {
    if field.is_null() {
        return Script::default();
    }

    let code_hash = field["code_hash"]
        .as_str()
        .expect(&format!("{} is missing", field_name));
    let code_hash_bytes = if let Some(caps) = RE_VARIABLE.captures(code_hash) {
        let cap = caps.get(1).expect("The captures[1] should always exist.");
        get_type_id_bytes(cap.as_str())
    } else {
        hex_to_bytes(code_hash)
    };

    let hash_type = match field["hash_type"].as_str() {
        Some("data") => ScriptHashType::Data,
        _ => ScriptHashType::Type,
    };
    let args = match field["args"].as_str() {
        Some(val) => hex_to_bytes(val),
        _ => Vec::new(),
    };

    Script::new_builder()
        .code_hash(Hash::try_from(code_hash_bytes).unwrap().into())
        .hash_type(Byte::new(hash_type as u8))
        .args(Bytes::from(args))
        .build()
}

/// Parse string in JSON to account ID bytes
///
/// It support both 0x-prefixed hex string and account name, if it is an account name, its ID will be calculated automatically.
pub fn parse_json_str_to_account_id(field_name: &str, field: &Value) -> Vec<u8> {
    let hex_or_str = parse_json_str(field_name, field);
    let id = if hex_or_str.starts_with("0x") {
        hex_to_bytes(hex_or_str)
    } else {
        account_to_id(hex_or_str)
    };

    id
}

/// Parse string in JSON to molecule struct AccountId
///
/// It support both 0x-prefixed hex string and account name, if it is an account name, its ID will be calculated automatically.
pub fn parse_json_str_to_account_id_mol(field_name: &str, field: &Value) -> AccountId {
    let account_id_bytes = parse_json_str_to_account_id(field_name, field);
    AccountId::try_from(account_id_bytes).expect(&format!("{} should be a 20 bytes hex string", field_name))
}

/// Parse records array in JSON to molecule struct Records
///
/// Example:
/// ```json
/// [
///     {
///         "type": "xxxxx",
///         "key": ""yyyyy,
///         "label": "zzzzz",
///         "value": "0x...",
///         "ttl": null | u32
///     },
///     ...
/// ]
/// ```
pub fn parse_json_to_records_mol(field_name: &str, field: &Value) -> Records {
    if field.is_null() {
        return Records::default();
    };

    let records = parse_json_array(field_name, field);
    let mut records_builder = Records::new_builder();
    for (_i, record) in records.iter().enumerate() {
        let record = Record::new_builder()
            .record_type(Bytes::from(
                parse_json_str(&format!("{}[].type", field_name), &record["type"]).as_bytes(),
            ))
            .record_key(Bytes::from(
                parse_json_str(&format!("{}[].key", field_name), &record["key"]).as_bytes(),
            ))
            .record_label(Bytes::from(
                parse_json_str(&format!("{}[].label", field_name), &record["label"]).as_bytes(),
            ))
            .record_value(Bytes::from(parse_json_hex(
                "cell.witness.records[].value",
                &record["value"],
            )))
            .record_ttl(Uint32::from(parse_json_u32(
                &format!("{}[].ttl", field_name),
                &record["ttl"],
                Some(300),
            )))
            .build();
        records_builder = records_builder.push(record);
    }

    records_builder.build()
}

pub fn parse_json_to_account_chars(
    field_name: &str,
    field: &Value,
    suffix_opt: Option<&str>,
) -> (String, AccountChars) {
    let suffix = if let Some(suffix) = suffix_opt { suffix } else { ".bit" };

    let mut account;
    let account_chars;
    if field.is_string() {
        // Parse the field as a string
        account = parse_json_str(field_name, field).to_string();
        let account_without_suffix = match account.strip_suffix(suffix) {
            Some(val) => val,
            _ => &account,
        };
        let account_chars_raw = account_without_suffix
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        account_chars = gen_account_chars(account_chars_raw);
    } else {
        // Parse the field as an AccountChars array.
        // Example:
        // ```json
        // [
        //     { char: "", type: u32 },
        //     { char: "", type: u32 },
        //     ...
        // ]
        // ```
        //
        // gen_account_char(char: &str, char_set_type: CharSetType)
        let json_chars = parse_json_array(field_name, field);
        let mut builder = AccountChars::new_builder();
        for json_char in json_chars.iter() {
            let char = parse_json_str(&format!("{}[].char", field_name), &json_char["char"]);
            let char_set_type = parse_json_u32(&format!("{}[].type", field_name), &json_char["type"], None);
            builder = builder.push(gen_account_char(
                char,
                CharSetType::try_from(char_set_type)
                    .expect(&format!("{} should only contain valid CharSetType.", field_name)),
            ));
        }
        account_chars = builder.build();
        account = String::from_utf8(account_chars.as_readable())
            .expect(&format!("{} should only contain UTF-8 characters.", field_name));
        account += suffix;
    }

    (account, account_chars)
}

pub fn gen_das_lock_args(owner_pubkey_hash: &str, manager_pubkey_hash_opt: Option<&str>) -> String {
    // TODO Unify format of args into one type.

    let owner_args;
    if owner_pubkey_hash.len() == 42 {
        owner_args = format!("00{}", owner_pubkey_hash.trim_start_matches("0x"));
    } else {
        owner_args = String::from(owner_pubkey_hash.trim_start_matches("0x"));
    }

    let manager_args;
    if let Some(manager_pubkey_hash) = manager_pubkey_hash_opt {
        if manager_pubkey_hash.len() == 42 {
            manager_args = format!("00{}", manager_pubkey_hash.trim_start_matches("0x"));
        } else {
            manager_args = String::from(manager_pubkey_hash.trim_start_matches("0x"));
        }
    } else {
        manager_args = owner_args.clone();
    }

    format!("0x{}{}", owner_args, manager_args)
}

/// Parse das-lock Script and fill optional fields
///
/// Example:
/// ```json
/// // input
/// {
///     "owner_lock_args": OWNER,
///     "manager_lock_args": MANAGER
/// }
/// // output
/// {
///     code_hash: "{{fake-das-lock}}",
///     hash_type: "type",
///     args: "0x..."
/// }
/// ```
pub fn parse_json_script_das_lock(field_name: &str, field: &Value) -> Value {
    if field.is_null() {
        panic!("{} is missing", field_name);
    }

    let owner_lock_args = parse_json_str(&format!("{}.owner_lock_args", field_name), &field["owner_lock_args"]);
    let manager_lock_args = parse_json_str(
        &format!("{}.manager_lock_args", field_name),
        &field["manager_lock_args"],
    );
    let args = gen_das_lock_args(owner_lock_args, Some(manager_lock_args));

    json!({
        "code_hash": "{{fake-das-lock}}",
        "hash_type": "type",
        "args": args
    })
}
