use super::{super::ckb_types_relay::*, constants::*, smt::*, util};
use ckb_testtool::ckb_hash::blake2b_256;
use das_types_std::{constants::*, packed::*, prelude::*, util as das_util, util::EntityWrapper};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryFrom, env, fs::OpenOptions, io::Write, str};

pub enum SubAccountActionType {
    Insert,
    Edit,
    Delete,
}

pub fn gen_fake_das_lock(lock_args: &str) -> Script {
    Script::new_builder()
        .code_hash(Hash::try_from(util::get_type_id_bytes("fake-das-lock")).unwrap())
        .hash_type(Byte::new(1))
        .args(Bytes::from(util::hex_to_bytes(lock_args)))
        .build()
}

pub fn gen_fake_signhash_all_lock(lock_args: &str) -> Script {
    Script::new_builder()
        .code_hash(Hash::try_from(util::get_type_id_bytes("fake-secp256k1-blake160-signhash-all")).unwrap())
        .hash_type(Byte::new(1))
        .args(Bytes::from(util::hex_to_bytes(lock_args)))
        .build()
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

fn gen_price_config(length: u8, new_price: u64, renew_price: u64) -> PriceConfig {
    PriceConfig::new_builder()
        .length(Uint8::from(length))
        .new(Uint64::from(new_price))
        .renew(Uint64::from(renew_price))
        .build()
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

fn witness_bytes_to_hex(input: Bytes) -> String {
    "0x".to_string() + &hex::encode(input.as_reader().raw_data())
}

/// Parse u64 in JSON
///
/// Support both **number** and **string** format.
fn parse_json_u64(field_name: &str, field: &Value, default: Option<u64>) -> u64 {
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
fn parse_json_u32(field_name: &str, field: &Value, default: Option<u32>) -> u32 {
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
fn parse_json_u8(field_name: &str, field: &Value, default: Option<u8>) -> u8 {
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

/// Parse string in JSON
///
/// All string will be treated as utf8 encoding.
fn parse_json_str<'a>(field_name: &str, field: &'a Value) -> &'a str {
    field.as_str().expect(&format!("{} is missing", field_name))
}

fn parse_json_str_with_default<'a>(field_name: &str, field: &'a Value, default: &'a str) -> &'a str {
    if field.is_null() {
        default
    } else {
        parse_json_str(field_name, field)
    }
}

/// Parse string in JSON and return &[u8]
///
/// All string will be treated as utf8 encoding.
fn parse_json_str_to_bytes<'a>(field_name: &str, field: &'a Value) -> &'a [u8] {
    field.as_str().expect(&format!("{} is missing", field_name)).as_bytes()
}

/// Parse hex string in JSON
///
/// Prefix "0x" is optional.
fn parse_json_hex(field_name: &str, field: &Value) -> Vec<u8> {
    let mut hex = field.as_str().expect(&format!("{} is missing", field_name));
    hex = hex.trim_start_matches("0x");

    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect(&format!("{} is should be hex string", field_name))
    }
}

/// Parse hex string in JSON, if it is not exist return the default value.
fn parse_json_hex_with_default(field_name: &str, field: &Value, default: Vec<u8>) -> Vec<u8> {
    if field.is_null() {
        default
    } else {
        parse_json_hex(field_name, field)
    }
}

/// Parse array in JSON
fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a [Value] {
    field
        .as_array()
        .map(|v| v.as_slice())
        .expect(&format!("{} is missing", field_name))
}

/// Parse struct Script and fill optional fields
///
/// Example:
/// ```json
/// // input
/// {
///     code_hash: "{{xxx-cell-type}}"
///     hash_type: "type", // could be omit if it is "type"    
///     args: "" // could be omit if it it empty
/// }
/// // output
/// {
///     code_hash: "{{xxx-cell-type}}",
///     hash_type: "type",
///     args: ""
/// }
/// ```
fn parse_json_script(field_name: &str, field: &Value) -> Value {
    let code_hash = field["code_hash"]
        .as_str()
        .expect(&format!("{} is missing", field_name));
    let hash_type = match field["hash_type"].as_str() {
        Some("data") => "data",
        _ => "type",
    };
    let args = match field["args"].as_str() {
        Some(val) => val,
        _ => "",
    };

    json!({
        "code_hash": code_hash,
        "hash_type": hash_type,
        "args": args
    })
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
fn parse_json_script_to_mol(field_name: &str, field: &Value) -> Script {
    if field.is_null() {
        return Script::default();
    }

    let code_hash = field["code_hash"]
        .as_str()
        .expect(&format!("{} is missing", field_name));
    let code_hash_bytes = if let Some(caps) = RE_VARIABLE.captures(code_hash) {
        let cap = caps.get(1).expect("The captures[1] should always exist.");
        util::get_type_id_bytes(cap.as_str())
    } else {
        util::hex_to_bytes(code_hash)
    };

    let hash_type = match field["hash_type"].as_str() {
        Some("data") => ScriptHashType::Data,
        _ => ScriptHashType::Type,
    };
    let args = match field["args"].as_str() {
        Some(val) => util::hex_to_bytes(val),
        _ => Vec::new(),
    };

    Script::new_builder()
        .code_hash(Hash::try_from(code_hash_bytes).unwrap().into())
        .hash_type(Byte::new(hash_type as u8))
        .args(Bytes::from(args))
        .build()
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

/// Parse string in JSON to account ID bytes
///
/// It support both 0x-prefixed hex string and account name, if it is an account name, its ID will be calculated automatically.
fn parse_json_str_to_account_id(field_name: &str, field: &Value) -> Vec<u8> {
    let hex_or_str = parse_json_str(field_name, field);
    let id = if hex_or_str.starts_with("0x") {
        util::hex_to_bytes(hex_or_str)
    } else {
        util::account_to_id(hex_or_str)
    };

    id
}

/// Parse string in JSON to molecule struct AccountId
///
/// It support both 0x-prefixed hex string and account name, if it is an account name, its ID will be calculated automatically.
fn parse_json_str_to_account_id_mol(field_name: &str, field: &Value) -> AccountId {
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
fn parse_json_to_records_mol(field_name: &str, field: &Value) -> Records {
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

fn parse_json_to_account_chars(field_name: &str, field: &Value, suffix_opt: Option<&str>) -> (String, AccountChars) {
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

pub fn parse_json_to_sub_account(field_name: &str, field: &Value) -> SubAccount {
    // let lock = parse_json_script_das_lock(&format!("{}.lock", field_name), &field["lock"]);
    let lock = parse_json_script_to_mol(
        "",
        &parse_json_script_das_lock(&format!("{}.lock", field_name), &field["lock"]),
    );
    let suffix = parse_json_str(&format!("{}.suffix", field_name), &field["suffix"]);
    let (account, account_chars) =
        parse_json_to_account_chars(&format!("{}.account", field_name), &field["account"], Some(suffix));
    let account_id = if !field["id"].is_null() {
        parse_json_str_to_account_id_mol(&format!("{}.id", field_name), &field["id"])
    } else {
        AccountId::try_from(util::account_to_id(&account)).expect("Calculate account ID from account failed")
    };
    let registered_at = Uint64::from(parse_json_u64(
        &format!("{}.registered_at", field_name),
        &field["registered_at"],
        None,
    ));
    let expired_at = Uint64::from(parse_json_u64(
        &format!("{}.expired_at", field_name),
        &field["expired_at"],
        None,
    ));
    let status = Uint8::from(parse_json_u8(
        &format!("{}.status", field_name),
        &field["status"],
        Some(0),
    ));
    let records = parse_json_to_records_mol(&format!("{}.records", field_name), &field["records"]);
    let nonce = Uint64::from(parse_json_u64(
        &format!("{}.nonce", field_name),
        &field["nonce"],
        Some(0),
    ));
    let enable_sub_account = Uint8::from(parse_json_u8(
        &format!("{}.enable_sub_account", field_name),
        &field["enable_sub_account"],
        Some(0),
    ));
    let renew_sub_account_price = Uint64::from(parse_json_u64(
        &format!("{}.renew_sub_account_price", field_name),
        &field["renew_sub_account_price"],
        Some(0),
    ));

    SubAccount::new_builder()
        .lock(lock)
        .id(account_id)
        .account(account_chars)
        .suffix(Bytes::from(suffix.as_bytes()))
        .registered_at(registered_at)
        .expired_at(expired_at)
        .status(status)
        .records(records)
        .nonce(nonce)
        .enable_sub_account(enable_sub_account)
        .renew_sub_account_price(renew_sub_account_price)
        .build()
}

#[derive(Debug, Clone)]
pub struct AccountRecordParam {
    pub type_: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IncomeRecordParam {
    pub belong_to: String,
    pub capacity: u64,
}

pub struct TemplateGenerator {
    pub header_deps: Vec<Value>,
    pub cell_deps: Vec<Value>,
    pub inputs: Vec<Value>,
    pub outputs: Vec<Value>,
    pub inner_witnesses: Vec<String>,
    pub outer_witnesses: Vec<String>,
    pub sub_account_outer_witnesses: Vec<String>,
    pub prices: HashMap<u8, PriceConfig>,
    pub preserved_account_groups: HashMap<u32, (Vec<u8>, Vec<u8>)>,
    pub charsets: HashMap<u32, (Bytes, Vec<u8>)>,
    pub smt_with_history: SMTWithHistory,
}

impl TemplateGenerator {
    pub fn new(action: &str, params_opt: Option<Bytes>) -> TemplateGenerator {
        let witness = das_util::wrap_action_witness(action, params_opt);

        let mut prices = HashMap::new();
        prices.insert(1u8, gen_price_config(1, ACCOUNT_PRICE_1_CHAR, ACCOUNT_PRICE_1_CHAR));
        prices.insert(2u8, gen_price_config(2, ACCOUNT_PRICE_2_CHAR, ACCOUNT_PRICE_2_CHAR));
        prices.insert(3u8, gen_price_config(3, ACCOUNT_PRICE_3_CHAR, ACCOUNT_PRICE_3_CHAR));
        prices.insert(4u8, gen_price_config(4, ACCOUNT_PRICE_4_CHAR, ACCOUNT_PRICE_4_CHAR));
        prices.insert(5u8, gen_price_config(5, ACCOUNT_PRICE_5_CHAR, ACCOUNT_PRICE_5_CHAR));
        prices.insert(6u8, gen_price_config(6, ACCOUNT_PRICE_5_CHAR, ACCOUNT_PRICE_5_CHAR));
        prices.insert(7u8, gen_price_config(7, ACCOUNT_PRICE_5_CHAR, ACCOUNT_PRICE_5_CHAR));
        prices.insert(8u8, gen_price_config(8, ACCOUNT_PRICE_5_CHAR, ACCOUNT_PRICE_5_CHAR));

        TemplateGenerator {
            header_deps: Vec::new(),
            cell_deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            inner_witnesses: Vec::new(),
            outer_witnesses: vec![witness_bytes_to_hex(witness)],
            sub_account_outer_witnesses: Vec::new(),
            prices,
            preserved_account_groups: HashMap::new(),
            charsets: HashMap::new(),
            smt_with_history: SMTWithHistory::new(),
        }
    }

    pub fn push_witness_args(
        &mut self,
        lock_opt: Option<&[u8]>,
        input_type: Option<&[u8]>,
        output_type: Option<&[u8]>,
    ) {
        let mut witness_args_builder = witness_args_new_builder();

        if let Some(bytes) = lock_opt {
            witness_args_builder = witness_args_builder.lock(to_bytes_opt(bytes));
        }
        if let Some(bytes) = input_type {
            witness_args_builder = witness_args_builder.input_type(to_bytes_opt(bytes));
        }
        if let Some(bytes) = output_type {
            witness_args_builder = witness_args_builder.output_type(to_bytes_opt(bytes));
        }

        self.inner_witnesses
            .push(witness_bytes_to_hex(Bytes::from(to_slice(witness_args_build(
                witness_args_builder,
            )))));
    }

    pub fn push_empty_witness(&mut self) {
        self.inner_witnesses.push(String::from("0x"));
    }

    pub fn push_das_lock_witness(&mut self, type_data_hash_hex: &str) {
        let signature = util::hex_to_bytes("0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000FF");
        let type_data_hash = util::hex_to_bytes(type_data_hash_hex);
        let chain_id = util::hex_to_bytes("0x0000000000000001");
        let lock = [signature, type_data_hash, chain_id].concat();
        self.push_witness_args(Some(&lock), None, None);
    }

    pub fn push_multi_sign_witness(
        &mut self,
        require_first_n: u8,
        threshold: u8,
        sign_address_len: u8,
        args_hex: &str,
    ) {
        let args = util::hex_to_bytes(args_hex);
        let mut lock = vec![0, require_first_n, threshold, sign_address_len];
        lock.extend_from_slice(&args);

        self.push_witness_args(Some(&lock), None, None);
    }

    pub fn push_cell(
        &mut self,
        capacity: u64,
        lock_script: Value,
        type_script: Value,
        data: Option<Bytes>,
        source: Source,
    ) -> usize {
        let mut value;
        if let Some(tmp_data) = data {
            value = json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": witness_bytes_to_hex(tmp_data.clone())
            });
        } else {
            value = json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script
            });
        }

        if source == Source::Input {
            value = json!({
                "previous_output": value,
                "since": "0x"
            });
        }

        match source {
            Source::CellDep => {
                self.cell_deps.push(value);
                self.cell_deps.len() - 1
            }
            Source::Input => {
                self.inputs.push(value);
                self.inputs.len() - 1
            }
            Source::Output => {
                self.outputs.push(value);
                self.outputs.len() - 1
            }
            _ => panic!("Only CellDep, Input and Output are supported"),
        }
    }

    pub fn push_oracle_cell(&mut self, index: u8, type_: OracleCellType, data: u64) {
        let mut cell_raw_data = Vec::new();
        cell_raw_data.extend(index.to_be_bytes().iter());
        cell_raw_data.extend(&[type_ as u8]);
        cell_raw_data.extend(data.to_be_bytes().iter());
        let cell_data = Bytes::from(cell_raw_data);

        let lock_script = json!({
            "code_hash": "{{always_success}}"
        });
        let type_script = json!({
            "code_hash": "0x0100000000000000000000000000000000000000000000000000000000000000",
            "hash_type": "type",
            "args": format!("0x{}", hex::encode(&[type_ as u8]))
        });

        self.push_cell(
            40_000_000_000,
            lock_script,
            type_script,
            Some(cell_data),
            Source::CellDep,
        );
    }

    fn gen_config_cell_account(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellAccount::new_builder()
            .max_length(Uint32::from(42))
            .basic_capacity(Uint64::from(ACCOUNT_BASIC_CAPACITY))
            .prepared_fee_capacity(Uint64::from(ACCOUNT_PREPARED_FEE_CAPACITY))
            .expiration_grace_period(Uint32::from(ACCOUNT_EXPIRATION_GRACE_PERIOD as u32))
            .record_min_ttl(Uint32::from(300))
            .record_size_limit(Uint32::from(5000))
            .transfer_account_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_manager_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_records_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .common_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .transfer_account_throttle(Uint32::from(86400))
            .edit_manager_throttle(Uint32::from(3600))
            .edit_records_throttle(Uint32::from(600))
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellAccount(entity))
    }

    fn gen_config_cell_apply(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellApply::new_builder()
            .apply_min_waiting_block_number(Uint32::from(APPLY_MIN_WAITING_BLOCK as u32))
            .apply_max_waiting_block_number(Uint32::from(APPLY_MAX_WAITING_BLOCK as u32))
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellApply(entity))
    }

    fn gen_config_cell_income(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellIncome::new_builder()
            .basic_capacity(Uint64::from(INCOME_BASIC_CAPACITY))
            .max_records(Uint32::from(50))
            .min_transfer_capacity(Uint64::from(10_000_000_000))
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellIncome(entity))
    }

    fn gen_config_cell_main(&mut self) -> (Vec<u8>, EntityWrapper) {
        let type_id_table = TypeIdTable::new_builder()
            .account_cell(Hash::try_from(util::get_type_id_bytes("account-cell-type")).unwrap())
            .apply_register_cell(Hash::try_from(util::get_type_id_bytes("apply-register-cell-type")).unwrap())
            .account_sale_cell(Hash::try_from(util::get_type_id_bytes("account-sale-cell-type")).unwrap())
            .account_auction_cell(Hash::try_from(util::get_type_id_bytes("account-auction-cell-type")).unwrap())
            .balance_cell(Hash::try_from(util::get_type_id_bytes("balance-cell-type")).unwrap())
            .income_cell(Hash::try_from(util::get_type_id_bytes("income-cell-type")).unwrap())
            .offer_cell(Hash::try_from(util::get_type_id_bytes("offer-cell-type")).unwrap())
            .pre_account_cell(Hash::try_from(util::get_type_id_bytes("pre-account-cell-type")).unwrap())
            .proposal_cell(Hash::try_from(util::get_type_id_bytes("proposal-cell-type")).unwrap())
            .reverse_record_cell(Hash::try_from(util::get_type_id_bytes("reverse-record-cell-type")).unwrap())
            .sub_account_cell(Hash::try_from(util::get_type_id_bytes("sub-account-cell-type")).unwrap())
            .eip712_lib(Hash::try_from(util::get_type_id_bytes("eip712-lib")).unwrap())
            .build();

        let entity = ConfigCellMain::new_builder()
            .status(Uint8::from(1))
            .type_id_table(type_id_table)
            .das_lock_out_point_table(DasLockOutPointTable::default())
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellMain(entity))
    }

    fn gen_config_cell_price(&mut self) -> (Vec<u8>, EntityWrapper) {
        let discount_config = DiscountConfig::new_builder()
            .invited_discount(Uint32::from(INVITED_DISCOUNT as u32))
            .build();

        let mut prices = PriceConfigList::new_builder();
        for (_, price) in self.prices.iter() {
            prices = prices.push(price.to_owned());
        }

        let entity = ConfigCellPrice::new_builder()
            .discount(discount_config)
            .prices(prices.build())
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellPrice(entity))
    }

    fn gen_config_cell_proposal(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellProposal::new_builder()
            .proposal_min_confirm_interval(Uint8::from(4))
            .proposal_min_extend_interval(Uint8::from(2))
            .proposal_min_recycle_interval(Uint8::from(6))
            .proposal_max_account_affect(Uint32::from(50))
            .proposal_max_pre_account_contain(Uint32::from(50))
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellProposal(entity))
    }

    fn gen_config_cell_profit_rate(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellProfitRate::new_builder()
            .channel(Uint32::from(800))
            .inviter(Uint32::from(800))
            .proposal_create(Uint32::from(400))
            .proposal_confirm(Uint32::from(0))
            .income_consolidate(Uint32::from(CONSOLIDATING_FEE as u32))
            .sale_buyer_inviter(Uint32::from(SALE_BUYER_INVITER_PROFIT_RATE as u32))
            .sale_buyer_channel(Uint32::from(SALE_BUYER_CHANNEL_PROFIT_RATE as u32))
            .sale_das(Uint32::from(100))
            .auction_bidder_inviter(Uint32::from(100))
            .auction_bidder_channel(Uint32::from(100))
            .auction_das(Uint32::from(100))
            .auction_prev_bidder(Uint32::from(4700))
            .build();

        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellProfitRate(entity))
    }

    fn gen_config_cell_release(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellRelease::new_builder()
            .lucky_number(Uint32::from(3435973836))
            .build();
        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellRelease(entity))
    }

    fn gen_config_cell_secondary_market(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellSecondaryMarket::new_builder()
            .common_fee(Uint64::from(SECONDARY_MARKET_COMMON_FEE))
            .sale_min_price(Uint64::from(ACCOUNT_SALE_MIN_PRICE))
            .sale_expiration_limit(Uint32::from(86400 * 30))
            .sale_description_bytes_limit(Uint32::from(5000))
            .sale_cell_basic_capacity(Uint64::from(ACCOUNT_SALE_BASIC_CAPACITY))
            .sale_cell_prepared_fee_capacity(Uint64::from(ACCOUNT_SALE_PREPARED_FEE_CAPACITY))
            .auction_max_extendable_duration(Uint32::from(86400 * 7))
            .auction_duration_increment_each_bid(Uint32::from(600))
            .auction_min_opening_price(Uint64::from(200_000_000_000))
            .auction_min_increment_rate_each_bid(Uint32::from(1000))
            .auction_description_bytes_limit(Uint32::from(5000))
            .auction_cell_basic_capacity(Uint64::from(20_000_000_000))
            .auction_cell_prepared_fee_capacity(Uint64::from(100_000_000))
            .offer_min_price(Uint64::from(0))
            .offer_cell_basic_capacity(Uint64::from(OFFER_BASIC_CAPACITY))
            .offer_cell_prepared_fee_capacity(Uint64::from(OFFER_PREPARED_FEE_CAPACITY))
            .offer_message_bytes_limit(Uint32::from(OFFER_PREPARED_MESSAGE_BYTES_LIMIT as u32))
            .build();
        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellSecondaryMarket(entity))
    }

    fn gen_config_cell_reverse_resolution(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellReverseResolution::new_builder()
            .record_basic_capacity(Uint64::from(REVERSE_RECORD_BASIC_CAPACITY))
            .record_prepared_fee_capacity(Uint64::from(REVERSE_RECORD_PREPARED_FEE_CAPACITY))
            .common_fee(Uint64::from(REVERSE_RECORD_COMMON_FEE))
            .build();
        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellReverseResolution(entity))
    }

    fn gen_config_cell_sub_account(&mut self) -> (Vec<u8>, EntityWrapper) {
        let entity = ConfigCellSubAccount::new_builder()
            .basic_capacity(Uint64::from(SUB_ACCOUNT_BASIC_CAPACITY))
            .prepared_fee_capacity(Uint64::from(SUB_ACCOUNT_PREPARED_FEE_CAPACITY))
            .new_sub_account_price(Uint64::from(SUB_ACCOUNT_NEW_PRICE))
            .new_sub_account_custom_price_das_profit_rate(Uint32::from(
                SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE as u32,
            ))
            .renew_sub_account_price(Uint64::from(SUB_ACCOUNT_RENEW_PRICE))
            .renew_sub_account_custom_price_das_profit_rate(Uint32::from(
                SUB_ACCOUNT_RENEW_CUSTOM_PRICE_DAS_PROFIT_RATE as u32,
            ))
            .common_fee(Uint64::from(SUB_ACCOUNT_COMMON_FEE))
            .create_fee(Uint64::from(SUB_ACCOUNT_CREATE_FEE))
            .edit_fee(Uint64::from(SUB_ACCOUNT_EDIT_FEE))
            .renew_fee(Uint64::from(SUB_ACCOUNT_RENEW_FEE))
            .recycle_fee(Uint64::from(SUB_ACCOUNT_RECYCLE_FEE))
            .build();
        let cell_data = blake2b_256(entity.as_slice()).to_vec();

        (cell_data, EntityWrapper::ConfigCellSubAccount(entity))
    }

    fn gen_config_cell_record_key_namespace(&mut self) -> (Vec<u8>, Vec<u8>) {
        let mut record_key_namespace = Vec::new();
        let lines = util::read_lines("record_key_namespace.txt")
            .expect("Expect file ./tests/data/record_key_namespace.txt exist.");
        for line in lines {
            if let Ok(key) = line {
                record_key_namespace.push(key);
            }
        }
        record_key_namespace.sort();

        // Join all record keys with 0x00 byte as entity.
        let mut raw = Vec::new();
        for key in record_key_namespace {
            raw.extend(key.as_bytes());
            raw.extend(&[0u8]);
        }
        let raw = util::prepend_molecule_like_length(raw);

        let cell_data = blake2b_256(raw.as_slice()).to_vec();

        (cell_data, raw)
    }

    fn gen_config_cell_preserved_account(&mut self, data_type: DataType) -> Option<(Vec<u8>, Vec<u8>)> {
        if self.preserved_account_groups.is_empty() {
            // Load and group preserved accounts
            let mut preserved_accounts_groups: Vec<Vec<Vec<u8>>> =
                vec![Vec::new(); PRESERVED_ACCOUNT_CELL_COUNT as usize];
            let lines =
                util::read_lines("preserved_accounts.txt").expect("Expect file ./data/preserved_accounts.txt exist.");
            for line in lines {
                if let Ok(account) = line {
                    let account_hash = blake2b_256(account.as_bytes())
                        .get(..ACCOUNT_ID_LENGTH)
                        .unwrap()
                        .to_vec();
                    let index = (account_hash[0] % PRESERVED_ACCOUNT_CELL_COUNT) as usize;

                    preserved_accounts_groups[index].push(account_hash);
                }
            }

            // Store grouped preserved accounts into self.preserved_account_groups
            for (_i, mut group) in preserved_accounts_groups.into_iter().enumerate() {
                // println!("Preserved account group[{}] count: {}", _i, group.len());
                group.sort();
                let mut raw = group.into_iter().flatten().collect::<Vec<u8>>();
                raw = util::prepend_molecule_like_length(raw);

                let data_type = das_util::preserved_accounts_group_to_data_type(_i);
                let cell_data = blake2b_256(raw.as_slice()).to_vec();
                self.preserved_account_groups.insert(data_type as u32, (cell_data, raw));
            }
        }

        self.preserved_account_groups
            .get(&(data_type as u32))
            .map(|item| item.to_owned())
    }

    fn gen_config_cell_unavailable_account(&mut self) -> (Vec<u8>, Vec<u8>) {
        // Load and group unavailable accounts
        let mut unavailable_account_hashes = Vec::new();
        let lines = util::read_lines("unavailable_account_hashes.txt")
            .expect("Expect file ./tests/data/unavailable_account_hashes.txt exist.");

        for line in lines {
            if let Ok(account_hash_string) = line {
                let account_hash: Vec<u8> = hex::decode(account_hash_string).unwrap();
                unavailable_account_hashes.push(account_hash.get(..ACCOUNT_ID_LENGTH).unwrap().to_vec());
            }
        }

        unavailable_account_hashes.sort(); // todo: maybe we don't need to sort, traverse is just enough

        let mut raw = Vec::new();

        for account_hash in unavailable_account_hashes {
            raw.extend(account_hash);
        }
        let raw = util::prepend_molecule_like_length(raw);

        let cell_data = blake2b_256(raw.as_slice()).to_vec();

        (cell_data, raw)
    }

    fn gen_config_cell_char_set(&mut self, file_name: &str, is_global: u8) -> (Vec<u8>, Vec<u8>) {
        let mut charsets = Vec::new();
        let lines =
            util::read_lines(file_name).expect(format!("Expect file ./tests/data/{} exist.", file_name).as_str());
        for line in lines {
            if let Ok(key) = line {
                charsets.push(key);
            }
        }

        // Join all record keys with 0x00 byte as entity.
        let mut raw = Vec::new();
        raw.push(is_global); // global status
        for key in charsets {
            raw.extend(key.as_bytes());
            raw.extend(&[0u8]);
        }
        raw = util::prepend_molecule_like_length(raw);

        let cell_data = blake2b_256(raw.as_slice()).to_vec();

        (cell_data, raw)
    }

    fn gen_config_cell_sub_account_beta_list(&mut self) -> (Vec<u8>, Vec<u8>) {
        // Load and group unavailable accounts
        let mut sub_account_beta_list = Vec::new();
        let lines = util::read_lines("sub_account_beta_list.txt")
            .expect("Expect file ./tests/data/sub_account_beta_list.txt exist.");

        for line in lines {
            if let Ok(account) = line {
                let account_hash = blake2b_256(account.as_bytes())
                    .get(..ACCOUNT_ID_LENGTH)
                    .unwrap()
                    .to_vec();

                sub_account_beta_list.push(account_hash);
            }
        }

        sub_account_beta_list.sort();
        let mut raw = sub_account_beta_list.into_iter().flatten().collect::<Vec<u8>>();
        raw = util::prepend_molecule_like_length(raw);

        let cell_data = blake2b_256(raw.as_slice()).to_vec();

        (cell_data, raw)
    }

    pub fn push_config_cell(&mut self, config_type: DataType, source: Source) {
        fn push_cell(
            generator: &mut TemplateGenerator,
            config_type: DataType,
            outputs_data_bytes: Vec<u8>,
            source: Source,
        ) -> usize {
            let outputs_data = util::bytes_to_hex(&outputs_data_bytes);
            let config_id = String::from("0x") + &hex::encode(&(config_type as u32).to_le_bytes());
            let mut cell = json!({
                "tmp_type": "full",
                "capacity": 0,
                "lock": json!({
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": CONFIG_LOCK_ARGS
                }),
                "type": json!({
                    "code_hash": "{{config-cell-type}}",
                    "args": config_id,
                }),
                "tmp_data": outputs_data
            });

            if source == Source::Input {
                cell = json!({
                    "previous_output": cell,
                    "since": "0x"
                });
            }

            match source {
                Source::CellDep => {
                    generator.cell_deps.push(cell);
                    generator.cell_deps.len() - 1
                }
                Source::Input => {
                    generator.inputs.push(cell);
                    generator.inputs.len() - 1
                }
                Source::Output => {
                    generator.outputs.push(cell);
                    generator.outputs.len() - 1
                }
                _ => panic!("Only CellDep, Input and Output are supported"),
            }
        }

        // Create config cell.
        macro_rules! push_cell {
            (@entity $fn:ident) => {{
                let (outputs_data, entity) = self.$fn();
                push_cell(self, config_type, outputs_data, source);
                let witness = match source {
                    Source::Input => das_util::wrap_entity_witness_v3(config_type, entity),
                    Source::Output => das_util::wrap_entity_witness_v3(config_type, entity),
                    _ => das_util::wrap_entity_witness_v3(config_type, entity),
                };
                self.outer_witnesses.push(witness_bytes_to_hex(witness));
            }};
            (@raw $fn:ident) => {{
                let (outputs_data, raw) = self.$fn();
                push_cell(self, config_type, outputs_data, source);
                let witness = match source {
                    Source::Input => das_util::wrap_raw_witness(config_type, raw),
                    Source::Output => das_util::wrap_raw_witness(config_type, raw),
                    _ => das_util::wrap_raw_witness(config_type, raw),
                };
                self.outer_witnesses.push(witness_bytes_to_hex(witness));
            }};
            (@char_set $fn:ident, $file_name:expr, $is_global:expr) => {{
                let (outputs_data, raw) = self.$fn($file_name, $is_global);
                push_cell(self, config_type, outputs_data, source);
                let witness = match source {
                    Source::Input => das_util::wrap_raw_witness(config_type, raw),
                    Source::Output => das_util::wrap_raw_witness(config_type, raw),
                    _ => das_util::wrap_raw_witness(config_type, raw),
                };
                self.outer_witnesses.push(witness_bytes_to_hex(witness));
            }};
            (@preserved_account $fn:ident, $config_type:expr) => {{
                if let Some((outputs_data, raw)) = self.$fn($config_type) {
                    push_cell(self, config_type, outputs_data, source);
                    let witness = match source {
                        Source::Input => das_util::wrap_raw_witness(config_type, raw),
                        Source::Output => das_util::wrap_raw_witness(config_type, raw),
                        _ => das_util::wrap_raw_witness(config_type, raw),
                    };
                    self.outer_witnesses.push(witness_bytes_to_hex(witness));
                } else {
                    panic!("Load preserved_account failed.");
                }
            }};
        }

        match config_type {
            // ConfigCells with molecule encoding data.
            DataType::ConfigCellApply => push_cell!(@entity gen_config_cell_apply),
            DataType::ConfigCellIncome => push_cell!(@entity gen_config_cell_income),
            DataType::ConfigCellMain => push_cell!(@entity gen_config_cell_main),
            DataType::ConfigCellAccount => push_cell!(@entity gen_config_cell_account),
            DataType::ConfigCellPrice => push_cell!(@entity gen_config_cell_price),
            DataType::ConfigCellProposal => push_cell!(@entity gen_config_cell_proposal),
            DataType::ConfigCellProfitRate => push_cell!(@entity gen_config_cell_profit_rate),
            DataType::ConfigCellRelease => push_cell!(@entity gen_config_cell_release),
            DataType::ConfigCellSecondaryMarket => push_cell!(@entity gen_config_cell_secondary_market),
            DataType::ConfigCellReverseResolution => push_cell!(@entity gen_config_cell_reverse_resolution),
            DataType::ConfigCellSubAccount => push_cell!(@entity gen_config_cell_sub_account),
            // ConfigCells with raw binary data.
            DataType::ConfigCellRecordKeyNamespace => push_cell!(@raw gen_config_cell_record_key_namespace),
            DataType::ConfigCellUnAvailableAccount => push_cell!(@raw gen_config_cell_unavailable_account),
            DataType::ConfigCellCharSetEmoji => push_cell!(@char_set gen_config_cell_char_set, "char_set_emoji.txt", 1),
            DataType::ConfigCellCharSetDigit => {
                push_cell!(@char_set gen_config_cell_char_set, "char_set_digit_and_symbol.txt", 1)
            }
            DataType::ConfigCellCharSetEn => push_cell!(@char_set gen_config_cell_char_set, "char_set_en.txt", 0),
            DataType::ConfigCellCharSetJa => push_cell!(@char_set gen_config_cell_char_set, "char_set_ja.txt", 0),
            DataType::ConfigCellCharSetKo => push_cell!(@char_set gen_config_cell_char_set, "char_set_ko.txt", 0),
            DataType::ConfigCellCharSetRu => push_cell!(@char_set gen_config_cell_char_set, "char_set_ru.txt", 0),
            DataType::ConfigCellCharSetTh => push_cell!(@char_set gen_config_cell_char_set, "char_set_th.txt", 0),
            DataType::ConfigCellCharSetTr => push_cell!(@char_set gen_config_cell_char_set, "char_set_tr.txt", 0),
            DataType::ConfigCellCharSetVi => push_cell!(@char_set gen_config_cell_char_set, "char_set_vi.txt", 0),
            DataType::ConfigCellSubAccountBetaList => push_cell!(@raw gen_config_cell_sub_account_beta_list),
            DataType::ConfigCellCharSetZhHans => {
                push_cell!(@char_set gen_config_cell_char_set, "char_set_zh_hans.txt", 0)
            }
            DataType::ConfigCellCharSetZhHant => {
                push_cell!(@char_set gen_config_cell_char_set, "char_set_zh_hant.txt", 0)
            }
            DataType::ConfigCellPreservedAccount00 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount00)
            }
            DataType::ConfigCellPreservedAccount01 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount01)
            }
            DataType::ConfigCellPreservedAccount02 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount02)
            }
            DataType::ConfigCellPreservedAccount03 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount03)
            }
            DataType::ConfigCellPreservedAccount04 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount04)
            }
            DataType::ConfigCellPreservedAccount05 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount05)
            }
            DataType::ConfigCellPreservedAccount06 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount06)
            }
            DataType::ConfigCellPreservedAccount07 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount07)
            }
            DataType::ConfigCellPreservedAccount08 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount08)
            }
            DataType::ConfigCellPreservedAccount09 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount09)
            }
            DataType::ConfigCellPreservedAccount10 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount10)
            }
            DataType::ConfigCellPreservedAccount11 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount11)
            }
            DataType::ConfigCellPreservedAccount12 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount12)
            }
            DataType::ConfigCellPreservedAccount13 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount13)
            }
            DataType::ConfigCellPreservedAccount14 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount14)
            }
            DataType::ConfigCellPreservedAccount15 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount15)
            }
            DataType::ConfigCellPreservedAccount16 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount16)
            }
            DataType::ConfigCellPreservedAccount17 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount17)
            }
            DataType::ConfigCellPreservedAccount18 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount18)
            }
            DataType::ConfigCellPreservedAccount19 => {
                push_cell!(@preserved_account gen_config_cell_preserved_account, DataType::ConfigCellPreservedAccount19)
            }
            _ => panic!("Undefined config cell type."),
        }
    }

    pub fn push_config_cell_derived_by_account(&mut self, account: &str, source: Source) {
        let account_without_suffix = match account.strip_suffix(".bit") {
            Some(val) => val,
            _ => account,
        };
        let first_byte_of_account_hash = blake2b_256(account_without_suffix.as_bytes())[0];
        let index = (first_byte_of_account_hash % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
        let config_type = das_util::preserved_accounts_group_to_data_type(index);

        // println!(
        //     "The first byte of account hash is {:?}, so {:?} will be chosen.",
        //     first_byte_of_account_hash, config_type
        // );
        self.push_config_cell(config_type, source);
    }

    // ======

    pub fn push_contract_cell(&mut self, contract_filename: &str, deployed: bool) {
        let value;
        if deployed {
            value = json!({
                "tmp_type": "deployed_contract",
                "tmp_file_name": contract_filename
            });
        } else {
            value = json!({
                "tmp_type": "contract",
                "tmp_file_name": contract_filename
            });
        }

        self.cell_deps.push(value)
    }

    pub fn push_shared_lib_cell(&mut self, contract_filename: &str, deployed: bool) {
        let value;
        if deployed {
            value = json!({
                "tmp_type": "deployed_shared_lib",
                "tmp_file_name": contract_filename
            });
        } else {
            value = json!({
                "tmp_type": "shared_lib",
                "tmp_file_name": contract_filename
            });
        }

        self.cell_deps.push(value)
    }

    pub fn push_dep(&mut self, cell: Value, version_opt: Option<u32>) -> usize {
        self.push_cell_v2(cell, Source::CellDep, version_opt)
    }

    pub fn push_input(&mut self, cell: Value, version_opt: Option<u32>) -> usize {
        self.push_cell_v2(cell, Source::Input, version_opt)
    }

    pub fn push_output(&mut self, cell: Value, version_opt: Option<u32>) -> usize {
        self.push_cell_v2(cell, Source::Output, version_opt)
    }

    pub fn push_cell_v2(&mut self, cell: Value, source: Source, version_opt: Option<u32>) -> usize {
        macro_rules! push_cell {
            ($gen_fn:ident, $cell:expr) => {{
                let (cell, _) = self.$gen_fn($cell);
                self.push_cell_json(cell, source)
            }};
            ($data_type:expr, $gen_fn:ident, $version_opt:expr, $cell:expr) => {{
                let version = if let Some(version) = $version_opt {
                    version
                } else {
                    1
                };

                let (cell, entity_opt) = self.$gen_fn(version, $cell);
                self.push_cell_json_with_entity(cell, source, $data_type, version, entity_opt)
            }};
        }

        if let Some(type_script) = cell.get("type") {
            let code_hash = type_script
                .get("code_hash")
                .expect("cell.type.code_hash is missing")
                .as_str()
                .expect("cell.type.code_hash should be a string");

            if let Some(caps) = RE_VARIABLE.captures(code_hash) {
                let type_id = caps
                    .get(1)
                    .map(|m| m.as_str())
                    .expect("type.code_hash is something like '{{...}}'");
                let index = match type_id {
                    "account-cell-type" => {
                        push_cell!(DataType::AccountCellData, gen_account_cell, version_opt, cell)
                    }
                    "account-sale-cell-type" => {
                        push_cell!(DataType::AccountSaleCellData, gen_account_sale_cell, version_opt, cell)
                    }
                    "apply-register-cell-type" => push_cell!(gen_apply_register_cell, cell),
                    "balance-cell-type" => push_cell!(gen_balance_cell, cell),
                    "sub-account-cell-type" => push_cell!(gen_sub_account_cell, cell),
                    "income-cell-type" => {
                        push_cell!(DataType::IncomeCellData, gen_income_cell, version_opt, cell)
                    }
                    "offer-cell-type" => {
                        push_cell!(DataType::OfferCellData, gen_offer_cell, version_opt, cell)
                    }
                    "pre-account-cell-type" => {
                        push_cell!(DataType::PreAccountCellData, gen_pre_account_cell, version_opt, cell)
                    }
                    "proposal-cell-type" => {
                        push_cell!(DataType::ProposalCellData, gen_proposal_cell, version_opt, cell)
                    }
                    "reverse-record-cell-type" => push_cell!(gen_reverse_record_cell, cell),
                    "test-env" => push_cell!(gen_custom_cell, cell),
                    "playground" => push_cell!(gen_custom_cell, cell),
                    _ => panic!("Unknown type ID {}", type_id),
                };

                index
            } else {
                panic!("{}", "type.code_hash is something like '{{...}}'")
            }
        } else {
            push_cell!(gen_custom_cell, cell)
        }
    }

    pub fn push_cell_json(&mut self, mut cell: Value, source: Source) -> usize {
        if source == Source::Input {
            cell = json!({
                "previous_output": cell,
                "since": "0x"
            });
        }

        match source {
            Source::CellDep => {
                self.cell_deps.push(cell);
                self.cell_deps.len() - 1
            }
            Source::Input => {
                self.inputs.push(cell);
                self.inputs.len() - 1
            }
            Source::Output => {
                self.outputs.push(cell);
                self.outputs.len() - 1
            }
            _ => panic!("Only CellDep, Input and Output are supported"),
        }
    }

    pub fn push_cell_json_with_entity(
        &mut self,
        cell: Value,
        source: Source,
        data_type: DataType,
        version: u32,
        entity_opt: Option<EntityWrapper>,
    ) -> usize {
        let index = self.push_cell_json(cell, source);

        if let Some(entity) = entity_opt {
            let witness = das_util::wrap_data_witness_v3(data_type, version, index, entity, source);
            self.outer_witnesses.push(witness_bytes_to_hex(witness));
        }

        index
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
    ///         "args": "0x..."
    ///     },
    ///     "type": {
    ///         "code_hash": "{{apply-register-cell-type}}"
    ///     },
    ///     "data": {
    ///         "account": null | "xxxxx.bit", // If this is null, it will be an invalid cell.
    ///         "height": u64,
    ///         "timestamp": u64
    ///     }
    /// })
    /// ```
    fn gen_apply_register_cell(&mut self, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        let outputs_data = if cell["data"].is_null() {
            String::from("")
        } else {
            let data = &cell["data"];
            let mut raw = Vec::new();
            let mut account_hash_bytes = if data["account"].is_null() {
                Vec::new()
            } else {
                let account = parse_json_str("cell.data.account", &data["account"]);
                let lock_args = parse_json_hex("cell.lock.args", &lock_script["args"]);

                blake2b_256([&lock_args, account.as_bytes()].concat().as_slice()).to_vec()
            };

            let mut height = if data["height"].is_null() {
                Vec::new()
            } else {
                parse_json_u64("cell.data.height", &data["height"], None)
                    .to_le_bytes()
                    .to_vec()
            };
            let mut timestamp = if data["timestamp"].is_null() {
                Vec::new()
            } else {
                parse_json_u64("cell.data.timestamp", &data["timestamp"], None)
                    .to_le_bytes()
                    .to_vec()
            };

            raw.append(&mut account_hash_bytes);
            raw.append(&mut height);
            raw.append(&mut timestamp);
            util::bytes_to_hex(&raw)
        };

        (
            json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": outputs_data
            }),
            None,
        )
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "code_hash": "{{always_success}}",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{pre-account-cell-type}}"
    ///     },
    ///     "data": {
    ///         "hash": null | "0x...", // if this is null, will be calculated from witness.
    ///         "id": null | "0x..." // if this is null, will be calculated from account.
    ///     },
    ///     "witness": {
    ///         "account": "xxxxx.bit",
    ///         "refund_lock": Script,
    ///         "owner_lock_args": "0x...",
    ///         "inviter_id": "0x..." | null, // if this is null, will be Bytes::default().
    ///         "inviter_lock": Script | null, // if this is null, will be ScriptOpt::default().
    ///         "channel_lock": Script | null, // if this is null, will be ScriptOpt::default().
    ///         "price": {
    ///             "length": u8,
    ///             "new": u64,
    ///             "renew": u64
    ///         }
    ///         "quote": u64,
    ///         "invited_discount": u32,
    ///         "created_at": u64
    ///     }
    /// })
    /// ```
    fn gen_pre_account_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let (account, account_chars) =
                parse_json_to_account_chars("cell.witness.account", &witness["account"], None);
            let refund_lock = parse_json_script_to_mol("cell.witness.refund_lock", &witness["refund_lock"]);
            let owner_lock_args = parse_json_hex("cell.witness.owner_lock_args", &witness["owner_lock_args"]);
            let inviter_id = if !witness["inviter_id"].is_null() {
                Bytes::from(parse_json_hex("cell.witness.inviter_id", &witness["inviter_id"]))
            } else {
                Bytes::default()
            };
            let inviter_lock = if !witness["inviter_lock"].is_null() {
                ScriptOpt::from(parse_json_script_to_mol(
                    "cell.witness.inviter_lock",
                    &witness["inviter_lock"],
                ))
            } else {
                ScriptOpt::default()
            };
            let channel_lock = if !witness["channel_lock"].is_null() {
                ScriptOpt::from(parse_json_script_to_mol(
                    "cell.witness.inviter_lock",
                    &witness["channel_lock"],
                ))
            } else {
                ScriptOpt::default()
            };
            let price = PriceConfig::new_builder()
                .length(parse_json_u8("cell.witness.price.length", &witness["price"]["length"], None).into())
                .new(parse_json_u64("cell.witness.price.new", &witness["price"]["new"], None).into())
                .renew(parse_json_u64("cell.witness.price.renew", &witness["price"]["renew"], None).into())
                .build();
            let quote = parse_json_u64("cell.witness.quote", &witness["quote"], None);
            let invited_discount = parse_json_u32("cell.witness.invited_discount", &witness["invited_discount"], None);
            let created_at = parse_json_u64("cell.witness.created_at", &witness["created_at"], None);

            match version {
                1 => {
                    let entity = PreAccountCellDataV1::new_builder()
                        .account(account_chars)
                        .refund_lock(refund_lock)
                        .owner_lock_args(Bytes::from(owner_lock_args))
                        .inviter_id(inviter_id)
                        .inviter_lock(inviter_lock)
                        .channel_lock(channel_lock)
                        .price(price)
                        .quote(Uint64::from(quote))
                        .invited_discount(Uint32::from(invited_discount))
                        .created_at(Uint64::from(created_at))
                        .build();

                    let data = &cell["data"];
                    let hash = parse_json_hex_with_default(
                        "cell.data.hash",
                        &data["hash"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );
                    let account_id =
                        parse_json_hex_with_default("cell.data.id", &data["id"], util::account_to_id(&account));
                    let outputs_data = [hash, account_id].concat();

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::PreAccountCellDataV1(entity)),
                    )
                }
                _ => {
                    let initial_records =
                        parse_json_to_records_mol("cell.witness.initial_records", &witness["initial_records"]);
                    let entity = PreAccountCellData::new_builder()
                        .account(account_chars)
                        .refund_lock(refund_lock)
                        .owner_lock_args(Bytes::from(owner_lock_args))
                        .inviter_id(inviter_id)
                        .inviter_lock(inviter_lock)
                        .channel_lock(channel_lock)
                        .price(price)
                        .quote(Uint64::from(quote))
                        .invited_discount(Uint32::from(invited_discount))
                        .created_at(Uint64::from(created_at))
                        .initial_records(initial_records)
                        .build();

                    let data = &cell["data"];
                    let hash = parse_json_hex_with_default(
                        "cell.data.hash",
                        &data["hash"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );
                    let account_id =
                        parse_json_hex_with_default("cell.data.id", &data["id"], util::account_to_id(&account));
                    let outputs_data = [hash, account_id].concat();

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::PreAccountCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "code_hash": "{{always_success}}",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{proposal-cell-type}}"
    ///     },
    ///     "data": {
    ///         "hash": null | "0x..." // if this is null, will be calculated from witness.
    ///     },
    ///     "witness": {
    ///         "proposer_lock": Script,
    ///         "created_at_height": u64,
    ///         "slices": [
    ///             [
    ///                 {
    ///                     "account_id": "0x...",
    ///                     "item_type": u8,
    ///                     "next": "0x..."
    ///                 }
    ///             ]
    ///         ]
    ///     }
    /// })
    /// ```
    fn gen_proposal_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let proposer_lock = parse_json_script_to_mol("cell.witness.proposer_lock", &witness["proposer_lock"]);
            let created_at_height =
                parse_json_u64("cell.witness.created_at_height", &witness["created_at_height"], None);

            // The ProposalCellData.slices is a two-dimensional arrays.
            let mut slice_list_builder = SliceList::new_builder();
            if let Some(slices) = witness["slices"].as_array() {
                for (i, slice_val) in slices.iter().enumerate() {
                    let mut slice_builder = SL::new_builder();
                    if let Some(slice) = slice_val.as_array() {
                        for (j, item) in slice.iter().enumerate() {
                            let field_name_base = format!("cell.witness.slices[{}][{}]", i, j);
                            let account_id = parse_json_str_to_account_id_mol(
                                &format!("{}.account_id", field_name_base),
                                &item["account_id"],
                            );
                            let item_type =
                                parse_json_u8(&format!("{}.item_type", field_name_base), &item["item_type"], None);
                            let next =
                                parse_json_str_to_account_id_mol(&format!("{}.next", field_name_base), &item["next"]);

                            slice_builder = slice_builder.push(
                                ProposalItem::new_builder()
                                    .account_id(account_id)
                                    .item_type(Uint8::from(item_type))
                                    .next(next)
                                    .build(),
                            );
                        }
                    } else {
                        panic!("cell.witness.slices[{}] is missing.", i)
                    }
                    slice_list_builder = slice_list_builder.push(slice_builder.build());
                }
            } else {
                panic!("cell.witness.slices is missing.");
            }

            match version {
                _ => {
                    let entity = ProposalCellData::new_builder()
                        .proposer_lock(proposer_lock)
                        .created_at_height(Uint64::from(created_at_height))
                        .slices(slice_list_builder.build())
                        .build();
                    let outputs_data = parse_json_hex_with_default(
                        "cell.data",
                        &cell["data"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::ProposalCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "owner_lock_args": "0x...",
    ///         "manager_lock_args": "0x...",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{account-cell-type}}"
    ///     },
    ///     "data": {
    ///         "hash": null | "0x...", // If this is null, it will be calculated from witness.
    ///         "id": null | "xxxxx.bit" | "0x...", // If this is null, it will be calculated from account. If this is not hex, it will be treated as account to calculate account ID.
    ///         "next": "yyyyy.bit" | "0x...", // If this is not hex, it will be be treated as account to calculate account ID.
    ///         "expired_at": u64,
    ///         "account": "xxxxx.bit"
    ///     },
    ///     "witness": {
    ///         "id": null | "xxxxx.bit" | "0x...", // If this is null, it will be calculated from account. If this is not hex, it will be treated as account to calculate account ID.
    ///         "account": "xxxxx.bit",
    ///         "registered_at": u64,
    ///         "last_transfer_account_at": u64,
    ///         "last_edit_manager_at": u64,
    ///         "last_edit_records_at": u64,
    ///         "status": u8,
    ///         "records": null | [
    ///             {
    ///                 "type": "xxxxx",
    ///                 "key": ""yyyyy,
    ///                 "label": "zzzzz",
    ///                 "value": "0x...",
    ///                 "ttl": null | u32
    ///             }
    ///         ],
    ///         "enable_sub_account": u8, // only latest version
    ///         "renew_sub_account_price": u64 // only latest version
    ///     }
    /// })
    /// ```
    fn gen_account_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script_das_lock("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        fn gen_outputs_data<T: Entity>(cell: &Value, entity: Option<&T>) -> Vec<u8> {
            let data = &cell["data"];
            let hash = if !data["hash"].is_null() {
                parse_json_hex("cell.data.hash", &data["hash"])
            } else {
                blake2b_256(entity.expect("The eneity should not be None.").as_slice()).to_vec()
            };
            let account = parse_json_str("cell.data.account", &data["account"]);
            let account_id = if !data["id"].is_null() {
                parse_json_str_to_account_id("cell.data.id", &data["id"])
            } else {
                util::account_to_id(account)
            };
            let next_id = parse_json_str_to_account_id("cell.data.next", &data["next"]);
            let expired_at = parse_json_u64("cell.data.expired_at", &data["expired_at"], None);

            [
                hash,
                account_id,
                next_id,
                expired_at.to_le_bytes().to_vec(),
                account.as_bytes().to_vec(),
            ]
            .concat()
        }

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let (account, account_chars) =
                parse_json_to_account_chars("cell.witness.account", &witness["account"], None);
            let account_id = if !witness["id"].is_null() {
                parse_json_str_to_account_id_mol("cell.witness.id", &witness["id"])
            } else {
                AccountId::try_from(util::account_to_id(&account)).expect("Calculate account ID from account failed")
            };
            let registered_at = Uint64::from(parse_json_u64(
                "cell.witness.registered_at",
                &witness["registered_at"],
                None,
            ));
            let last_transfer_account_at = Uint64::from(parse_json_u64(
                "cell.witness.last_transfer_account_at",
                &witness["last_transfer_account_at"],
                Some(0),
            ));
            let last_edit_manager_at = Uint64::from(parse_json_u64(
                "cell.witness.last_edit_manager_at",
                &witness["last_edit_manager_at"],
                Some(0),
            ));
            let last_edit_records_at = Uint64::from(parse_json_u64(
                "cell.witness.last_edit_records_at",
                &witness["last_edit_records_at"],
                Some(0),
            ));
            let status = Uint8::from(parse_json_u8("cell.witness.status", &witness["status"], Some(0)));
            let records = parse_json_to_records_mol("cell.witness.records", &witness["records"]);

            match version {
                2 => {
                    let entity = AccountCellDataV2::new_builder()
                        .id(account_id)
                        .account(account_chars)
                        .registered_at(registered_at)
                        .last_transfer_account_at(last_transfer_account_at)
                        .last_edit_manager_at(last_edit_manager_at)
                        .last_edit_records_at(last_edit_records_at)
                        .status(status)
                        .records(records)
                        .build();
                    let outputs_data = gen_outputs_data(&cell, Some(&entity));

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::AccountCellDataV2(entity)),
                    )
                }
                _ => {
                    let enable_sub_account = Uint8::from(parse_json_u8(
                        "cell.witness.enable_sub_account",
                        &witness["enable_sub_account"],
                        Some(0),
                    ));
                    let renew_sub_account_price = Uint64::from(parse_json_u64(
                        "cell.witness.renew_sub_account_price",
                        &witness["renew_sub_account_price"],
                        Some(0),
                    ));

                    let entity = AccountCellData::new_builder()
                        .id(account_id)
                        .account(account_chars)
                        .registered_at(registered_at)
                        .last_transfer_account_at(last_transfer_account_at)
                        .last_edit_manager_at(last_edit_manager_at)
                        .last_edit_records_at(last_edit_records_at)
                        .status(status)
                        .records(records)
                        .enable_sub_account(enable_sub_account)
                        .renew_sub_account_price(renew_sub_account_price)
                        .build();
                    let outputs_data = gen_outputs_data(&cell, Some(&entity));

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::AccountCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = gen_outputs_data::<AccountCellData>(&cell, None);

            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "owner_lock_args": "0x...",
    ///         "manager_lock_args": "0x...",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{account-sale-cell-type}}"
    ///     },
    ///     "data": null | "0x...", // if this is null, will be calculated from witness.
    ///     "witness": {
    ///         "account_id": null | "0x...", // if this is null, will be calculated from account.
    ///         "account": "xxxx.bit",
    ///         "price": u64,
    ///         "description": "some utf8 string",
    ///         "buyer_inviter_profit_rate": u32, // only latest version
    ///         "started_at": u64
    ///     }
    /// })
    /// ```
    fn gen_account_sale_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script_das_lock("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let account = Bytes::from(parse_json_str_to_bytes("cell.witness.account", &witness["account"]));
            let account_id = if !witness["account_id"].is_null() {
                AccountId::try_from(parse_json_hex("cell.witness.account_id", &witness["account_id"]))
                    .expect("cell.witness.account_id should be [u8; 20]")
            } else {
                let hash = blake2b_256(account.as_reader().raw_data());
                AccountId::try_from(&hash[..20]).expect("Calculate account ID from account failed")
            };
            let price = Uint64::from(parse_json_u64("cell.witness.price", &witness["price"], None));
            let description = Bytes::from(parse_json_str_to_bytes(
                "cell.witness.description",
                &witness["description"],
            ));
            let started_at = Uint64::from(parse_json_u64("cell.witness.started_at", &witness["started_at"], None));

            match version {
                1 => {
                    let entity = AccountSaleCellDataV1::new_builder()
                        .account_id(account_id)
                        .account(account)
                        .price(price)
                        .description(description)
                        .started_at(started_at)
                        .build();
                    let outputs_data = blake2b_256(entity.as_slice()).to_vec();

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::AccountSaleCellDataV1(entity)),
                    )
                }
                _ => {
                    let buyer_inviter_profit_rate = Uint32::from(parse_json_u32(
                        "cell.witness.buyer_inviter_profit_rate",
                        &witness["buyer_inviter_profit_rate"],
                        Some(0),
                    ));

                    let entity = AccountSaleCellData::new_builder()
                        .account_id(account_id)
                        .account(account)
                        .price(price)
                        .description(description)
                        .started_at(started_at)
                        .buyer_inviter_profit_rate(buyer_inviter_profit_rate)
                        .build();
                    let outputs_data = parse_json_hex_with_default(
                        "cell.data",
                        &cell["data"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::AccountSaleCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);

            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64 | null, // if this is null, it will be calculated from sum of records.
    ///     "lock": {
    ///         "code_hash": "{{always_success}}",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{income-cell-type}}"
    ///     },
    ///     "data": null | "0x...", // if this is null, it will be calculated from witness.
    ///     "witness": {
    ///         "creator": null | Script, // if this is null, it will be filled with Script::default().
    ///         "records": [
    ///             {
    ///                 "belong_to": Script,
    ///                 "capacity": u64
    ///             },
    ///             {
    ///                 "belong_to": Script,
    ///                 "capacity": u64
    ///             },
    ///             ...
    ///         ]
    ///     }
    /// })
    /// ```
    fn gen_income_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let creator = parse_json_script_to_mol("cell.witness.creator", &witness["creator"]);
            let mut records_builder = IncomeRecords::new_builder();

            let mut capacity_of_records = 0;
            if let Some(records) = witness["records"].as_array() {
                for (i, item) in records.iter().enumerate() {
                    let belong_to =
                        parse_json_script_to_mol(&format!("cell.winess.records[{}].belong_to", i), &item["belong_to"]);
                    let capacity =
                        parse_json_u64(&format!("cell.winess.records[{}].capacity", i), &item["capacity"], None);

                    capacity_of_records += capacity;
                    records_builder = records_builder.push(
                        IncomeRecord::new_builder()
                            .belong_to(belong_to)
                            .capacity(Uint64::from(capacity))
                            .build(),
                    );
                }
            }
            let capacity = parse_json_u64("cell.capacity", &cell["capacity"], Some(capacity_of_records));

            match version {
                _ => {
                    let entity = IncomeCellData::new_builder()
                        .creator(creator)
                        .records(records_builder.build())
                        .build();
                    let outputs_data = parse_json_hex_with_default(
                        "cell.data",
                        &cell["data"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );

                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::IncomeCellData(entity)),
                    )
                }
            }
        } else {
            let capacity = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64 | null, // if this is null, will be calculated from sum of records.
    ///     "lock": {
    ///         "owner_lock_args": "0x...",
    ///         "manager_lock_args": "0x...",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{reverse-record-cell-type}}"
    ///     },
    ///     "data": {
    ///         "account": null | "xxxx.bit" // It is possible to create an invalid cell without account.
    ///     }
    /// })
    /// ```
    fn gen_reverse_record_cell(&mut self, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script_das_lock("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        let outputs_data = if cell["data"].is_null() || cell["data"]["account"].is_null() {
            String::from("")
        } else {
            let account = parse_json_str("cell.data.account", &cell["data"]["account"]);
            util::bytes_to_hex(account.as_bytes())
        };

        (
            json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": outputs_data
            }),
            None,
        )
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64 | null, // if this is null, will be calculated from sum of records.
    ///     "lock": {
    ///         "owner_lock_args": "0x...",
    ///         "manager_lock_args": "0x...",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{offer-cell-type}}"
    ///     },
    ///     "data": null | "0x...", // if this is null, will be calculated from witness.
    ///     "witness": {
    ///         "account": "xxxx.bit",
    ///         "price": u64,
    ///         "message": "some utf8 string",
    ///         "inviter_lock": Script,
    ///         "channel_lock": Script
    ///     }
    /// })
    /// ```
    fn gen_offer_cell(&mut self, version: u32, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script_das_lock("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let account = parse_json_str("cell.witness.account", &witness["account"]);
            let price = parse_json_u64("cell.witness.price", &witness["price"], None);
            let message = parse_json_str("cell.witness.message", &witness["message"]);
            let inviter_lock = parse_json_script_to_mol("cell.witness.inviter_lock", &witness["inviter_lock"]);
            let channel_lock = parse_json_script_to_mol("cell.witness.channel_lock", &witness["channel_lock"]);

            match version {
                _ => {
                    let entity = OfferCellData::new_builder()
                        .account(Bytes::from(account.as_bytes()))
                        .price(Uint64::from(price))
                        .message(Bytes::from(message.as_bytes()))
                        .inviter_lock(inviter_lock)
                        .channel_lock(channel_lock)
                        .build();
                    let outputs_data = parse_json_hex_with_default(
                        "cell.data",
                        &cell["data"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );
                    (
                        json!({
                          "tmp_type": "full",
                          "capacity": capacity,
                          "lock": lock_script,
                          "type": type_script,
                          "tmp_data": util::bytes_to_hex(&outputs_data)
                        }),
                        Some(EntityWrapper::OfferCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (
                json!({
                  "tmp_type": "full",
                  "capacity": capacity,
                  "lock": lock_script,
                  "type": type_script,
                  "tmp_data": util::bytes_to_hex(&outputs_data)
                }),
                None,
            )
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "code_hash": "{{always_success}}",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{sub-account-cell-type}}",
    ///         "args": null | "xxxxx.bit" | "0x...", // If this is null, it will be an invalid cell. If this is not hex, it will be treated as account to calculate account ID.
    ///     },
    ///     "data": {
    ///         "root": null | "0x..." // If this is null, it will be an invalid cell.
    ///         "das_profit": 0,
    ///         "owner_profit": 0,
    ///         "custom_script": null | "0x...",
    ///         "script_args": null | "0x...",
    ///     }
    /// })
    /// ```
    fn gen_sub_account_cell(&mut self, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);

        let type_script = match cell.get("type") {
            Some(type_) => {
                let args = if type_["args"].is_null() {
                    String::from("0x")
                } else {
                    util::bytes_to_hex(&parse_json_str_to_account_id("cell.type.args", &type_["args"]))
                };

                json!({
                    "code_hash": type_["code_hash"],
                    "hash_type": "type",
                    "args": args
                })
            }
            _ => panic!("cell.type is missing"),
        };

        let outputs_data = if cell["data"].is_null() {
            String::from("")
        } else {
            let data = &cell["data"];
            let mut root = parse_json_hex("cell.data.root", &data["root"]);
            let mut das_profit = if data["das_profit"].is_null() {
                Vec::new()
            } else {
                parse_json_u64("cell.data.das_profit", &data["das_profit"], None)
                    .to_le_bytes()
                    .to_vec()
            };
            let mut owner_profit = if data["owner_profit"].is_null() {
                Vec::new()
            } else {
                parse_json_u64("cell.data.owner_profit", &data["owner_profit"], None)
                    .to_le_bytes()
                    .to_vec()
            };
            let mut custom_script =
                parse_json_hex_with_default("cell.data.custom_script", &data["custom_script"], Vec::new());
            let mut script_args =
                parse_json_hex_with_default("cell.data.script_args", &data["script_args"], Vec::new());

            root.append(&mut das_profit);
            root.append(&mut owner_profit);
            root.append(&mut custom_script);
            root.append(&mut script_args);
            util::bytes_to_hex(&root)
        };

        (
            json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": outputs_data
            }),
            None,
        )
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": {
    ///         "owner_lock_args": "0x...",
    ///         "manager_lock_args": "0x...",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{balance-cell-type}}"
    ///     },
    ///     "data": null | "0x..."
    /// })
    /// ```
    fn gen_balance_cell(&mut self, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script_das_lock("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        let outputs_data = if !cell["data"].is_null() {
            util::bytes_to_hex(&parse_json_hex("cell.data", &cell["data"]))
        } else {
            String::from("0x")
        };

        (
            json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": outputs_data
            }),
            None,
        )
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": Script,
    ///     "type": null | Script,
    ///     "data": null | "0x..."
    /// })
    /// ```
    fn gen_custom_cell(&mut self, cell: Value) -> (Value, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = cell["type"].clone();
        let outputs_data = if !cell["data"].is_null() {
            util::bytes_to_hex(&parse_json_hex("cell.data", &cell["data"]))
        } else {
            String::from("0x")
        };

        (
            json!({
              "tmp_type": "full",
              "capacity": capacity,
              "lock": lock_script,
              "type": type_script,
              "tmp_data": outputs_data
            }),
            None,
        )
    }

    /// Insert some leaves into the sparse-merkle-tree without pushing any witness
    pub fn restore_sub_account(&mut self, sub_account_jsons: Vec<Value>) {
        use sparse_merkle_tree::H256;

        let mut leaves: Vec<(H256, H256)> = Vec::new();

        for sub_account_json in sub_account_jsons {
            let account = parse_json_str("", &sub_account_json["account"]);
            let key = util::blake2b_smt(account.as_bytes());
            let sub_account_1 = parse_json_to_sub_account("", &sub_account_json);
            let value = util::blake2b_smt(sub_account_1.as_slice());
            leaves.push((key.into(), value.into()));
        }

        self.smt_with_history.restore_state(leaves);
    }

    /// Push sub-account witness
    ///
    /// The sub-account witnesses will always be the end of the whole witnesses array, so no matter when you need to call this function, you
    /// can call it freely.
    ///
    /// Witness structure:
    ///
    /// ```json
    /// json!({
    ///     "signature": null | "0x...", // If this is null, it will be filled with 65 bytes of 0.
    ///     "sign_role": 0 | 1, // 0 means owner, 1 means manager.
    ///     "prev_root": null | "0x...", // If this is null, it will be calculated automatically from self.smt_with_history.
    ///     "current_root": null | "0x...", // If this is null, it will be calculated automatically from self.smt_with_history.
    ///     "proof": "0x...", // If this is null, it will be calculated automatically from self.smt_with_history.
    ///     "version": u32,
    ///     "sub_account": {
    ///         "lock": Script,
    ///         "id": null | "yyyyy.xxxxx.bit" | "0x...", // If this is null, it will be an invalid cell. If this is not hex, it will be treated as account to calculate account ID.
    ///         "account": "yyyyy.xxxxx.bit",
    ///         "suffix": ".xxxxx.bit",
    ///         "registered_at": u64,
    ///         "expired_at": u64,
    ///         "status": u8,
    ///         "records": null | [
    ///             {
    ///                 "type": "xxxxx",
    ///                 "key": ""yyyyy,
    ///                 "label": "zzzzz",
    ///                 "value": "0x...",
    ///                 "ttl": null | u32
    ///             },
    ///             ...
    ///         ],
    ///         "nonce": u32,
    ///         "enable_sub_account": u8,
    ///         "renew_sub_account_price": u64
    ///     },
    ///     "edit_key": null | "expired_at",
    ///     "edit_value": null | ..., // A JSON object which expired_at
    /// })
    /// ```
    pub fn push_sub_account_witness(&mut self, action: SubAccountActionType, witness: Value) {
        fn length_of(data: &[u8]) -> Vec<u8> {
            (data.len() as u32).to_le_bytes().to_vec()
        }

        fn extend_main_fields(
            witness_bytes: &mut Vec<u8>,
            prev_root: [u8; 32],
            current_root: [u8; 32],
            proof: Vec<u8>,
            sub_account_entity_bytes: Vec<u8>,
            witness: &Value,
        ) {
            witness_bytes.extend(length_of(&prev_root));
            witness_bytes.extend(prev_root);

            witness_bytes.extend(length_of(&current_root));
            witness_bytes.extend(current_root);

            witness_bytes.extend(length_of(&proof));
            witness_bytes.extend(proof);

            let version = parse_json_u32("witness.version", &witness["version"], Some(1)).to_le_bytes();
            witness_bytes.extend(length_of(&version));
            witness_bytes.extend(version);

            witness_bytes.extend(length_of(&sub_account_entity_bytes));
            witness_bytes.extend(sub_account_entity_bytes);
        }

        fn extend_edit_fields(witness_bytes: &mut Vec<u8>, witness: &Value) {
            if witness["edit_key"].is_null() {
                witness_bytes.extend(length_of(&[]));
            } else {
                let edit_key = parse_json_str("witness.edit_key", &witness["edit_key"]);
                witness_bytes.extend(length_of(edit_key.as_bytes()));
                witness_bytes.extend(edit_key.as_bytes().to_vec());
            }

            if witness["edit_value"].is_null() {
                witness_bytes.extend(length_of(&[]));
            } else {
                // Allow the edit_key field to be an invalid value.
                let edit_key = parse_json_str_with_default("witness.edit_key", &witness["edit_key"], "");
                let edit_value = match edit_key {
                    "expired_at" => {
                        let mol = Uint64::from(parse_json_u64("witness.edit_value", &witness["edit_value"], None));
                        mol.as_slice().to_vec()
                    }
                    "owner" => parse_json_hex("witness.edit_value", &witness["edit_value"]),
                    "manager" => parse_json_hex("witness.edit_value", &witness["edit_value"]),
                    "records" => {
                        let mol = parse_json_to_records_mol("witness.edit_value", &witness["edit_value"]);
                        mol.as_slice().to_vec()
                    }
                    // If the edit_key field is invalid just parse edit_value field as hex string.
                    _ => parse_json_hex("witness.edit_value", &witness["edit_value"]),
                };
                witness_bytes.extend(length_of(&edit_value));
                witness_bytes.extend(edit_value);
            }
        }

        let mut witness_bytes = Vec::new();

        let field_value = parse_json_hex_with_default(
            "witness.signature",
            &witness["signature"],
            hex::decode("ffffffffffffffffffffffffffffffffffffffff").unwrap(),
        );
        witness_bytes.extend(length_of(&field_value));
        witness_bytes.extend(field_value);

        let field_value = parse_json_hex_with_default("witness.sign_role", &witness["sign_role"], vec![0]);
        witness_bytes.extend(length_of(&field_value));
        witness_bytes.extend(field_value);

        if witness["sub_account"].is_null() {
            panic!("witness.sub_account is missing");
        }
        let sub_account_value = &witness["sub_account"];
        let suffix = parse_json_str("witness.sub_account.suffix", &sub_account_value["suffix"]);
        let (account, _) = parse_json_to_account_chars(
            "witness.sub_account.account",
            &sub_account_value["account"],
            Some(suffix),
        );
        let key = util::blake2b_smt(account.as_bytes());

        let sub_account_entity = parse_json_to_sub_account("witness.sub_account", &witness["sub_account"]);

        match action {
            SubAccountActionType::Insert => {
                let sub_account_entity_bytes = sub_account_entity.as_slice().to_vec();
                let value = util::blake2b_smt(&sub_account_entity_bytes);
                let (prev_root, current_root, proof) = self.smt_with_history.insert(key.into(), value.into());

                extend_main_fields(
                    &mut witness_bytes,
                    prev_root,
                    current_root,
                    proof,
                    sub_account_entity_bytes,
                    &witness,
                );
                extend_edit_fields(&mut witness_bytes, &witness);
            }
            SubAccountActionType::Edit => {
                let mut new_sub_account_builder = sub_account_entity.clone().as_builder();
                let current_nonce = u64::from(sub_account_entity.nonce());

                // Modify SubAccount base on edit_key and edit_value.
                let edit_key = parse_json_str("witness.edit_key", &witness["edit_key"]);
                match edit_key {
                    "expired_at" => {
                        let mol = Uint64::from(parse_json_u64("witness.edit_value", &witness["edit_value"], None));
                        new_sub_account_builder = new_sub_account_builder.expired_at(mol)
                    }
                    "owner" | "manager" => {
                        let mut lock_builder = sub_account_entity.lock().as_builder();
                        let args = parse_json_hex("witness.edit_value", &witness["edit_value"]);
                        lock_builder = lock_builder.args(Bytes::from(args));

                        new_sub_account_builder = new_sub_account_builder.lock(lock_builder.build())
                    }
                    "records" => {
                        let mol = parse_json_to_records_mol("witness.edit_value", &witness["edit_value"]);
                        new_sub_account_builder = new_sub_account_builder.records(mol)
                    }
                    _ => panic!("Unsupported type of witness.edit_key !"),
                };

                new_sub_account_builder = new_sub_account_builder.nonce(Uint64::from(current_nonce + 1));
                let new_sub_account_entity = new_sub_account_builder.build();
                let new_sub_account_entity_bytes = new_sub_account_entity.as_slice().to_vec();
                let value = util::blake2b_smt(&new_sub_account_entity_bytes);
                let (prev_root, current_root, proof) = self.smt_with_history.insert(key.into(), value.into());

                extend_main_fields(
                    &mut witness_bytes,
                    prev_root,
                    current_root,
                    proof,
                    sub_account_entity.as_slice().to_vec(),
                    &witness,
                );
                extend_edit_fields(&mut witness_bytes, &witness);
            }
            SubAccountActionType::Delete => {
                todo!();
                // extend_edit_fields(&mut witness_bytes, &witness);
            }
        }

        witness_bytes = das_util::wrap_sub_account_witness(witness_bytes);
        self.sub_account_outer_witnesses
            .push(String::from("0x") + &hex::encode(&witness_bytes));
    }

    // ======

    pub fn as_json(&self) -> serde_json::Value {
        let witnesses = [
            self.inner_witnesses.clone(),
            self.outer_witnesses.clone(),
            self.sub_account_outer_witnesses.clone(),
        ]
        .concat();
        json!({
            "cell_deps": self.cell_deps,
            "inputs": self.inputs,
            "outputs": self.outputs,
            "witnesses": witnesses,
        })
    }

    pub fn write_template(&self, filename: &str) {
        let mut filepath = env::current_dir().unwrap();
        filepath.push("templates");
        filepath.push(filename);

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(filepath.clone())
            .expect(format!("Expect file path {:?} to be writable.", filepath).as_str());

        let data = serde_json::to_string_pretty(&self.as_json()).unwrap();
        file.write(data.as_bytes()).expect("Write file failed.");
    }
}
