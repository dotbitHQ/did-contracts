use super::super::ckb_types_relay::*;
use super::{constants::*, util};
use ckb_testtool::ckb_hash::blake2b_256;
use das_types_std::{constants::*, packed::*, prelude::*, util as das_util, util::EntityWrapper};
use hex;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryFrom, env, fs::OpenOptions, io::Write, str};

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

pub fn gen_account_record(type_: &str, key: &str, label: &str, value: impl AsRef<[u8]>, ttl: u32) -> Record {
    Record::new_builder()
        .record_type(Bytes::from(type_.as_bytes()))
        .record_key(Bytes::from(key.as_bytes()))
        .record_label(Bytes::from(label.as_bytes()))
        .record_value(Bytes::from(value.as_ref()))
        .record_ttl(Uint32::from(ttl))
        .build()
}

pub fn gen_account_records(records_param: Vec<AccountRecordParam>) -> Records {
    let mut records = Records::new_builder();
    for record_param in records_param.into_iter() {
        records = records.push(gen_account_record(
            record_param.type_,
            record_param.key,
            record_param.label,
            record_param.value,
            300,
        ));
    }
    records.build()
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

fn bytes_to_hex(input: Bytes) -> String {
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
fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a Vec<Value> {
    field.as_array().expect(&format!("{} is missing", field_name))
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

macro_rules! gen_config_cell_char_set {
    ($fn_name:ident, $is_global:expr, $file_name:expr, $ret_type:expr) => {
        fn $fn_name(&self) -> (Bytes, Vec<u8>) {
            let mut charsets = Vec::new();
            let lines =
                util::read_lines($file_name).expect(format!("Expect file ./tests/data/{} exist.", $file_name).as_str());
            for line in lines {
                if let Ok(key) = line {
                    charsets.push(key);
                }
            }

            // Join all record keys with 0x00 byte as entity.
            let mut raw = Vec::new();
            raw.push($is_global); // global status
            for key in charsets {
                raw.extend(key.as_bytes());
                raw.extend(&[0u8]);
            }
            raw = util::prepend_molecule_like_length(raw);

            let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());

            (cell_data, raw)
        }
    };
}

pub struct TemplateGenerator {
    pub header_deps: Vec<Value>,
    pub cell_deps: Vec<Value>,
    pub inputs: Vec<Value>,
    pub outputs: Vec<Value>,
    pub inner_witnesses: Vec<String>,
    pub outer_witnesses: Vec<String>,
    pub prices: HashMap<u8, PriceConfig>,
    pub preserved_account_groups: HashMap<u32, (Bytes, Vec<u8>)>,
    pub charsets: HashMap<u32, (Bytes, Vec<u8>)>,
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
            outer_witnesses: vec![bytes_to_hex(witness)],
            prices,
            preserved_account_groups: HashMap::new(),
            charsets: HashMap::new(),
        }
    }

    pub fn get_price(&self, account_length: usize) -> &PriceConfig {
        let key = if account_length > 8 { 8u8 } else { account_length as u8 };
        self.prices.get(&key).unwrap()
    }

    pub fn push_witness_with_group<T: Entity>(&mut self, data_type: DataType, group: Source, entity: (u32, u32, T)) {
        let witness = match group {
            Source::Input => das_util::wrap_data_witness::<T, T, T>(data_type, None, Some(entity), None),
            Source::Output => das_util::wrap_data_witness::<T, T, T>(data_type, Some(entity), None, None),
            _ => das_util::wrap_data_witness::<T, T, T>(data_type, None, None, Some(entity)),
        };
        self.outer_witnesses.push(bytes_to_hex(witness));
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
            .push(bytes_to_hex(Bytes::from(to_slice(witness_args_build(
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
              "tmp_data": bytes_to_hex(tmp_data.clone())
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

    pub fn push_apply_register_cell(
        &mut self,
        lock_args: &str,
        account: &str,
        height: u64,
        timestamp: u64,
        capacity: u64,
        source: Source,
    ) {
        let hash_of_account = Hash::new_unchecked(
            blake2b_256([&util::hex_to_bytes(lock_args), account.as_bytes()].concat().as_slice())
                .to_vec()
                .into(),
        );

        let raw = [
            hash_of_account.as_reader().raw_data(),
            &height.to_le_bytes(),
            &timestamp.to_le_bytes(),
        ]
        .concat();
        let cell_data = Bytes::from(raw);

        let lock_script = json!({
            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
            "args": lock_args
        });
        let type_script = json!({
            "code_hash": "{{apply-register-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);
    }

    fn gen_config_cell_account(&mut self) -> (Bytes, ConfigCellAccount) {
        let entity = ConfigCellAccount::new_builder()
            .max_length(Uint32::from(42))
            .basic_capacity(Uint64::from(ACCOUNT_BASIC_CAPACITY))
            .prepared_fee_capacity(Uint64::from(ACCOUNT_PREPARED_FEE_CAPACITY))
            .expiration_grace_period(Uint32::from(2_592_000))
            .record_min_ttl(Uint32::from(300))
            .record_size_limit(Uint32::from(5000))
            .transfer_account_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_manager_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .edit_records_fee(Uint64::from(ACCOUNT_OPERATE_FEE))
            .transfer_account_throttle(Uint32::from(86400))
            .edit_manager_throttle(Uint32::from(3600))
            .edit_records_throttle(Uint32::from(600))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_apply(&mut self) -> (Bytes, ConfigCellApply) {
        let entity = ConfigCellApply::new_builder()
            .apply_min_waiting_block_number(Uint32::from(1))
            .apply_max_waiting_block_number(Uint32::from(5760))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_income(&mut self) -> (Bytes, ConfigCellIncome) {
        let entity = ConfigCellIncome::new_builder()
            .basic_capacity(Uint64::from(20_000_000_000))
            .max_records(Uint32::from(50))
            .min_transfer_capacity(Uint64::from(10_000_000_000))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_main(&mut self) -> (Bytes, ConfigCellMain) {
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
            .build();

        let entity = ConfigCellMain::new_builder()
            .status(Uint8::from(1))
            .type_id_table(type_id_table)
            .das_lock_out_point_table(DasLockOutPointTable::default())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_price(&mut self) -> (Bytes, ConfigCellPrice) {
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

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_proposal(&mut self) -> (Bytes, ConfigCellProposal) {
        let entity = ConfigCellProposal::new_builder()
            .proposal_min_confirm_interval(Uint8::from(4))
            .proposal_min_extend_interval(Uint8::from(2))
            .proposal_min_recycle_interval(Uint8::from(6))
            .proposal_max_account_affect(Uint32::from(50))
            .proposal_max_pre_account_contain(Uint32::from(50))
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_profit_rate(&mut self) -> (Bytes, ConfigCellProfitRate) {
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

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_release(&mut self) -> (Bytes, ConfigCellRelease) {
        let data = vec![
            (
                2,
                util::gen_timestamp("2021-06-28 00:00:00"),
                util::gen_timestamp("2021-07-10 00:00:00"),
            ),
            (
                0,
                util::gen_timestamp("2021-06-01 00:00:00"),
                util::gen_timestamp("2021-06-01 00:00:00"),
            ),
        ];

        let mut release_rules = ReleaseRules::new_builder();
        for item in data.into_iter() {
            release_rules = release_rules.push(
                ReleaseRule::new_builder()
                    .length(Uint32::from(item.0))
                    .release_start(Uint64::from(item.1))
                    .release_end(Uint64::from(item.2))
                    .build(),
            );
        }

        let entity = ConfigCellRelease::new_builder()
            .release_rules(release_rules.build())
            .build();
        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_secondary_market(&mut self) -> (Bytes, ConfigCellSecondaryMarket) {
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
        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_reverse_resolution(&mut self) -> (Bytes, ConfigCellReverseResolution) {
        let entity = ConfigCellReverseResolution::new_builder()
            .record_basic_capacity(Uint64::from(REVERSE_RECORD_BASIC_CAPACITY))
            .record_prepared_fee_capacity(Uint64::from(REVERSE_RECORD_PREPARED_FEE_CAPACITY))
            .common_fee(Uint64::from(REVERSE_RECORD_COMMON_FEE))
            .build();
        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    fn gen_config_cell_record_key_namespace(&mut self) -> (Bytes, Vec<u8>) {
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

        let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());

        (cell_data, raw)
    }

    fn gen_config_cell_preserved_account(&mut self, data_type: DataType) -> Option<(Bytes, Vec<u8>)> {
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
                let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());
                self.preserved_account_groups.insert(data_type as u32, (cell_data, raw));
            }
        }

        self.preserved_account_groups
            .get(&(data_type as u32))
            .map(|item| item.to_owned())
    }

    fn gen_config_cell_unavailable_account(&mut self) -> (Bytes, Vec<u8>) {
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

        let cell_data = Bytes::from(blake2b_256(raw.as_slice()).to_vec());

        (cell_data, raw)
    }

    gen_config_cell_char_set!(
        gen_config_cell_char_set_emoji,
        1,
        "char_set_emoji.txt",
        DataType::ConfigCellCharSetEmoji
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_digit,
        1,
        "char_set_digit.txt",
        DataType::ConfigCellCharSetDigit
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_en,
        1,
        "char_set_en.txt",
        DataType::ConfigCellCharSetEn
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_zh_hans,
        1,
        "char_set_zh_hans.txt",
        DataType::ConfigCellCharSetZhHans
    );

    gen_config_cell_char_set!(
        gen_config_cell_char_set_zh_hant,
        1,
        "char_set_zh_hant.txt",
        DataType::ConfigCellCharSetZhHant
    );

    pub fn push_config_cell(&mut self, config_type: DataType, push_witness: bool, capacity: u64, source: Source) {
        macro_rules! match_entity {
            ( $var:expr, { $($type:pat => $gen_fn:ident, $config_type:expr),+ } ) => {
                match $var {
                    $($type => {
                        let (cell_data, entity) = self.$gen_fn();
                        Some((cell_data, das_util::wrap_entity_witness($config_type, entity)))
                    }),*
                    _ => None
                }
            };
        }

        macro_rules! match_raw {
            ( $var:expr, { $($type:pat => $gen_fn:ident, $config_type:expr),+ } ) => {
                match $var {
                    $($type => {
                        let (cell_data, raw) = self.$gen_fn();
                        Some((cell_data, das_util::wrap_raw_witness($config_type, raw)))
                    }),*
                    _ => None
                }
            };
        }

        // 646173700000001c0000000c0000001400000000c817a8040000008096980000000000
        let cell_data;
        let witness;
        if let Some((_data, _witness)) = match_entity!(config_type, {
            DataType::ConfigCellApply => gen_config_cell_apply, DataType::ConfigCellApply,
            DataType::ConfigCellIncome => gen_config_cell_income, DataType::ConfigCellIncome,
            DataType::ConfigCellMain => gen_config_cell_main, DataType::ConfigCellMain,
            DataType::ConfigCellAccount => gen_config_cell_account, DataType::ConfigCellAccount,
            DataType::ConfigCellPrice => gen_config_cell_price, DataType::ConfigCellPrice,
            DataType::ConfigCellProposal => gen_config_cell_proposal, DataType::ConfigCellProposal,
            DataType::ConfigCellProfitRate => gen_config_cell_profit_rate, DataType::ConfigCellProfitRate,
            DataType::ConfigCellRelease => gen_config_cell_release, DataType::ConfigCellRelease,
            DataType::ConfigCellSecondaryMarket => gen_config_cell_secondary_market, DataType::ConfigCellSecondaryMarket,
            DataType::ConfigCellReverseResolution => gen_config_cell_reverse_resolution, DataType::ConfigCellReverseResolution
        }) {
            cell_data = _data;
            witness = _witness;
        } else {
            if let Some((_data, _witness)) = match_raw!(config_type, {
                DataType::ConfigCellCharSetEmoji => gen_config_cell_char_set_emoji, DataType::ConfigCellCharSetEmoji,
                DataType::ConfigCellCharSetDigit => gen_config_cell_char_set_digit, DataType::ConfigCellCharSetDigit,
                DataType::ConfigCellCharSetEn => gen_config_cell_char_set_en, DataType::ConfigCellCharSetEn,
                DataType::ConfigCellCharSetZhHans => gen_config_cell_char_set_zh_hans, DataType::ConfigCellCharSetZhHans,
                DataType::ConfigCellCharSetZhHant => gen_config_cell_char_set_zh_hant, DataType::ConfigCellCharSetZhHant,
                DataType::ConfigCellRecordKeyNamespace => gen_config_cell_record_key_namespace, DataType::ConfigCellRecordKeyNamespace,
                DataType::ConfigCellUnAvailableAccount => gen_config_cell_unavailable_account, DataType::ConfigCellUnAvailableAccount
            }) {
                cell_data = _data;
                witness = _witness;
            } else {
                let (_data, _witness) = match config_type {
                    DataType::ConfigCellPreservedAccount00
                    | DataType::ConfigCellPreservedAccount01
                    | DataType::ConfigCellPreservedAccount02
                    | DataType::ConfigCellPreservedAccount03
                    | DataType::ConfigCellPreservedAccount04
                    | DataType::ConfigCellPreservedAccount05
                    | DataType::ConfigCellPreservedAccount06
                    | DataType::ConfigCellPreservedAccount07
                    | DataType::ConfigCellPreservedAccount08
                    | DataType::ConfigCellPreservedAccount09
                    | DataType::ConfigCellPreservedAccount10
                    | DataType::ConfigCellPreservedAccount11
                    | DataType::ConfigCellPreservedAccount12
                    | DataType::ConfigCellPreservedAccount13
                    | DataType::ConfigCellPreservedAccount14
                    | DataType::ConfigCellPreservedAccount15
                    | DataType::ConfigCellPreservedAccount16
                    | DataType::ConfigCellPreservedAccount17
                    | DataType::ConfigCellPreservedAccount18
                    | DataType::ConfigCellPreservedAccount19 => {
                        match self.gen_config_cell_preserved_account(config_type) {
                            Some((cell_data, raw)) => (cell_data, das_util::wrap_raw_witness(config_type, raw)),
                            None => panic!("Load preserved accounts from file failed ..."),
                        }
                    }
                    _ => {
                        panic!("Not config cell data type.")
                    }
                };
                cell_data = _data;
                witness = _witness;
            }
        }

        // Create config cell.
        let config_id_hex = hex::encode(&(config_type as u32).to_le_bytes());
        let lock_script = json!({
          "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
          "args": CONFIG_LOCK_ARGS
        });
        let type_script = json!({
          "code_hash": "{{config-cell-type}}",
          "args": format!("0x{}", config_id_hex),
        });
        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if push_witness {
            // Create config cell witness.
            self.outer_witnesses.push(bytes_to_hex(witness));
        }
    }

    pub fn push_config_cell_derived_by_account(
        &mut self,
        account_without_suffix: &str,
        push_witness: bool,
        capacity: u64,
        source: Source,
    ) {
        let first_byte_of_account_hash = blake2b_256(account_without_suffix.as_bytes())[0];
        let index = (first_byte_of_account_hash % PRESERVED_ACCOUNT_CELL_COUNT) as usize;
        let config_type = das_util::preserved_accounts_group_to_data_type(index);

        // println!(
        //     "The first byte of account hash is {:?}, so {:?} will be chosen.",
        //     first_byte_of_account_hash, config_type
        // );
        self.push_config_cell(config_type, push_witness, capacity, source);
    }

    pub fn gen_pre_account_cell_data(
        &mut self,
        account: &str,
        refund_lock_args: &str,
        owner_lock_args: &str,
        inviter_lock_args: &str,
        channel_lock_args: &str,
        quote: u64,
        invited_discount: u64,
        created_at: u64,
    ) -> (Bytes, PreAccountCellData) {
        let account_chars_raw = account[..account.len() - 4]
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        let account_chars = gen_account_chars(account_chars_raw);
        let account_length = if account_chars.len() > 8 {
            8u8
        } else {
            account_chars.len() as u8
        };

        let price = self.prices.get(&account_length).unwrap();
        let owner_lock_args = Bytes::from(util::hex_to_bytes(&gen_das_lock_args(owner_lock_args, None)));
        let mut entity_builder = PreAccountCellData::new_builder()
            .account(account_chars.to_owned())
            .owner_lock_args(owner_lock_args)
            .refund_lock(gen_fake_signhash_all_lock(refund_lock_args))
            .channel_lock(ScriptOpt::from(gen_fake_signhash_all_lock(channel_lock_args)))
            .price(price.to_owned())
            .quote(Uint64::from(quote))
            .invited_discount(Uint32::from(invited_discount as u32))
            .created_at(Uint64::from(created_at));

        if inviter_lock_args.is_empty() {
            entity_builder = entity_builder.inviter_lock(ScriptOpt::default());
            entity_builder = entity_builder.inviter_id(Bytes::default());
        } else {
            entity_builder =
                entity_builder.inviter_lock(ScriptOpt::from(gen_fake_signhash_all_lock(inviter_lock_args)));
            entity_builder = entity_builder.inviter_id(Bytes::from(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
        }

        if channel_lock_args.is_empty() {
            entity_builder = entity_builder.channel_lock(ScriptOpt::default());
        } else {
            entity_builder =
                entity_builder.channel_lock(ScriptOpt::from(gen_fake_signhash_all_lock(channel_lock_args)));
        }

        let entity = entity_builder.build();
        let id = util::account_to_id(account);

        let hash = Hash::try_from(blake2b_256(entity.as_slice()).to_vec()).unwrap();
        let raw = [hash.as_reader().raw_data(), id.as_slice()].concat();
        let cell_data = Bytes::from(raw);

        (cell_data, entity)
    }

    pub fn push_pre_account_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, impl Entity)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{pre-account-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::PreAccountCellData, source, entity);
        }
    }

    // TODO Refactor functions to support more flexible params
    pub fn gen_income_cell_data(
        &mut self,
        creator: &str,
        records_param: Vec<IncomeRecordParam>,
    ) -> (Bytes, IncomeCellData) {
        let creator = gen_fake_signhash_all_lock(creator);

        let mut records = IncomeRecords::new_builder();
        for record_param in records_param.into_iter() {
            records = records.push(
                IncomeRecord::new_builder()
                    .belong_to(gen_fake_signhash_all_lock(record_param.belong_to.as_str()))
                    .capacity(Uint64::from(record_param.capacity))
                    .build(),
            );
        }

        let entity = IncomeCellData::new_builder()
            .creator(creator)
            .records(records.build())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    pub fn gen_income_cell_data_with_das_lock(
        &mut self,
        creator: &str,
        records_param: Vec<IncomeRecordParam>,
    ) -> (Bytes, IncomeCellData) {
        let creator = gen_fake_das_lock(creator);

        let mut records = IncomeRecords::new_builder();
        for record_param in records_param.into_iter() {
            records = records.push(
                IncomeRecord::new_builder()
                    .belong_to(gen_fake_das_lock(record_param.belong_to.as_str()))
                    .capacity(Uint64::from(record_param.capacity))
                    .build(),
            );
        }

        let entity = IncomeCellData::new_builder()
            .creator(creator)
            .records(records.build())
            .build();

        let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());

        (cell_data, entity)
    }

    pub fn push_income_cell(
        &mut self,
        cell_data: Bytes,
        entity_opt: Option<(u32, u32, IncomeCellData)>,
        capacity: u64,
        source: Source,
    ) {
        let lock_script = json!({
          "code_hash": "{{always_success}}"
        });
        let type_script = json!({
          "code_hash": "{{income-cell-type}}"
        });

        self.push_cell(capacity, lock_script, type_script, Some(cell_data), source);

        if let Some(entity) = entity_opt {
            self.push_witness_with_group(DataType::IncomeCellData, source, entity);
        }
    }

    pub fn push_signall_cell(&mut self, lock_args: &str, capacity: u64, source: Source) {
        let lock_script = json!({
          "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
          "args": lock_args
        });

        self.push_cell(capacity, lock_script, json!(null), None, source);
        self.push_empty_witness();
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
                let (capacity, lock_script, type_script, outputs_data) = self.$gen_fn($cell);

                let outputs_data_bytes = Bytes::from(outputs_data);
                let index = self.push_cell(
                    capacity,
                    lock_script,
                    type_script,
                    Some(outputs_data_bytes),
                    source,
                );

                index
            }};
        }

        macro_rules! push_cell_with_witness {
            ($data_type:expr, $gen_fn:ident, $version_opt:expr, $cell:expr) => {{
                let version = if let Some(version) = $version_opt {
                    version
                } else {
                    1
                };

                let (capacity, lock_script, type_script, outputs_data, entity_opt) = self.$gen_fn(version, $cell);

                let outputs_data_bytes = Bytes::from(outputs_data);
                let index = self.push_cell(
                    capacity,
                    lock_script,
                    type_script,
                    Some(outputs_data_bytes),
                    source,
                );

                if let Some(entity) = entity_opt {
                    let index = index as u32;
                    let witness = match source {
                        Source::Input => {
                            das_util::wrap_data_witness_v3($data_type, version, index, entity, Source::Input)
                        }
                        Source::Output => {
                            das_util::wrap_data_witness_v3($data_type, version, index, entity, Source::Output)
                        }
                        _ => das_util::wrap_data_witness_v3($data_type, version, index, entity, Source::CellDep),
                    };
                    self.outer_witnesses.push(bytes_to_hex(witness));
                }

                index
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
                        push_cell_with_witness!(DataType::AccountCellData, gen_account_cell, version_opt, cell)
                    }
                    "account-sale-cell-type" => {
                        push_cell_with_witness!(DataType::AccountSaleCellData, gen_account_sale_cell, version_opt, cell)
                    }
                    "balance-cell-type" => {
                        let (capacity, lock_script, type_script, outputs_data_opt) = self.gen_balance_cell(cell);
                        let outputs_data_bytes_opt = if let Some(outputs_data) = outputs_data_opt {
                            Some(Bytes::from(outputs_data))
                        } else {
                            None
                        };

                        self.push_cell(capacity, lock_script, type_script, outputs_data_bytes_opt, source)
                    }
                    "income-cell-type" => {
                        push_cell_with_witness!(DataType::IncomeCellData, gen_income_cell, version_opt, cell)
                    }
                    "offer-cell-type" => {
                        push_cell_with_witness!(DataType::OfferCellData, gen_offer_cell, version_opt, cell)
                    }
                    "pre-account-cell-type" => {
                        push_cell_with_witness!(DataType::PreAccountCellData, gen_pre_account_cell, version_opt, cell)
                    }
                    "proposal-cell-type" => {
                        push_cell_with_witness!(DataType::ProposalCellData, gen_proposal_cell, version_opt, cell)
                    }
                    "reverse-record-cell-type" => push_cell!(gen_reverse_record_cell, cell),
                    "playground" => {
                        let (capacity, lock_script, type_script, outputs_data_opt) = self.gen_custom_cell(cell);
                        let outputs_data_bytes_opt = if let Some(outputs_data) = outputs_data_opt {
                            Some(Bytes::from(outputs_data))
                        } else {
                            None
                        };

                        self.push_cell(capacity, lock_script, type_script, outputs_data_bytes_opt, source)
                    }
                    _ => panic!("Unknown type ID {}", type_id),
                };

                index
            } else {
                panic!("{}", "type.code_hash is something like '{{...}}'")
            }
        } else {
            let (capacity, lock_script, type_script, outputs_data_opt) = self.gen_custom_cell(cell);
            let outputs_data_bytes_opt = if let Some(outputs_data) = outputs_data_opt {
                Some(Bytes::from(outputs_data))
            } else {
                None
            };

            let index = self.push_cell(capacity, lock_script, type_script, outputs_data_bytes_opt, source);

            index
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
    fn gen_pre_account_cell(
        &mut self,
        version: u32,
        cell: Value,
    ) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = parse_json_script("cell.type", &cell["type"]);

        if !cell["witness"].is_null() {
            let witness = &cell["witness"];
            let account = parse_json_str("cell.witness.account", &witness["account"]);
            let account_chars_raw = account
                .chars()
                .take(account.len() - 4)
                .map(|c| c.to_string())
                .collect::<Vec<String>>();
            let account_chars = gen_account_chars(account_chars_raw);
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
                _ => {
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
                        .build();

                    let data = &cell["data"];
                    let hash = parse_json_hex_with_default(
                        "cell.data.hash",
                        &data["hash"],
                        blake2b_256(entity.as_slice()).to_vec(),
                    );
                    let account_id =
                        parse_json_hex_with_default("cell.data.id", &data["id"], util::account_to_id(account));
                    let outputs_data = [hash, account_id].concat();

                    (
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::PreAccountCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (capacity, lock_script, type_script, outputs_data, None)
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
    fn gen_proposal_cell(&mut self, version: u32, cell: Value) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
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
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::ProposalCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (capacity, lock_script, type_script, outputs_data, None)
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
    ///         "hash": null | "0x...", // if this is null, will be calculated from witness.
    ///         "id": null | "0x...", // if this is null, will be calculated from account.
    ///         "next": "yyyyy.bit" | "0x...", // if this is not hex, will be calculated automatically.
    ///         "expired_at": u64,
    ///         "account": "xxxxx.bit"
    ///     },
    ///     "witness": {
    ///         "id": null | "0x...", // if this is null, will be calculated from account.
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
    fn gen_account_cell(&mut self, version: u32, cell: Value) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock = cell.get("lock").expect("cell.lock is missing");
        let owner_lock_args = parse_json_str("cell.lock.owner_lock_args", &lock["owner_lock_args"]);
        let manager_lock_args = parse_json_str("cell.lock.manager_lock_args", &lock["manager_lock_args"]);
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": gen_das_lock_args(owner_lock_args, Some(manager_lock_args))
        });

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
                parse_json_hex("cell.data.id", &data["id"])
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
            let account = parse_json_str("cell.witness.account", &witness["account"]);
            let account_id = if !witness["id"].is_null() {
                AccountId::try_from(parse_json_hex("cell.witness.id", &witness["id"]))
                    .expect("cell.witness.account_id should be [u8; 20]")
            } else {
                AccountId::try_from(util::account_to_id(account)).expect("Calculate account ID from account failed")
            };
            let account_chars_raw = account
                .chars()
                .take(account.len() - 4)
                .map(|c| c.to_string())
                .collect::<Vec<String>>();
            let account_chars = gen_account_chars(account_chars_raw);
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
            // TODO Find the correct way to handle the return type of &Vec.
            let tmp = Vec::new();
            let mut records = &tmp;
            if !witness["records"].is_null() {
                records = parse_json_array("cell.witness.records", &witness["records"])
            };

            let mut records_builder = Records::new_builder();
            for (_i, record) in records.iter().enumerate() {
                let ttl = if !record["ttl"].is_null() {
                    Uint32::from(parse_json_u32("cell.witness.records[].ttl", &record["ttl"], Some(300)))
                } else {
                    Uint32::from(300)
                };
                let record = Record::new_builder()
                    .record_type(Bytes::from(
                        parse_json_str("cell.witness.records[].type", &record["type"]).as_bytes(),
                    ))
                    .record_key(Bytes::from(
                        parse_json_str("cell.witness.records[].key", &record["key"]).as_bytes(),
                    ))
                    .record_label(Bytes::from(
                        parse_json_str("cell.witness.records[].label", &record["label"]).as_bytes(),
                    ))
                    .record_value(Bytes::from(parse_json_hex(
                        "cell.witness.records[].value",
                        &record["value"],
                    )))
                    .record_ttl(ttl)
                    .build();
                records_builder = records_builder.push(record);
            }

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
                        .records(records_builder.build())
                        .build();
                    let outputs_data = gen_outputs_data(&cell, Some(&entity));

                    (
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
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
                        .records(records_builder.build())
                        .enable_sub_account(enable_sub_account)
                        .renew_sub_account_price(renew_sub_account_price)
                        .build();
                    let outputs_data = gen_outputs_data(&cell, Some(&entity));

                    (
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::AccountCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = gen_outputs_data::<AccountCellData>(&cell, None);

            (capacity, lock_script, type_script, outputs_data, None)
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
    fn gen_account_sale_cell(
        &mut self,
        version: u32,
        cell: Value,
    ) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock = cell.get("lock").expect("cell.lock is missing");
        let owner_lock_args = parse_json_str("cell.lock.owner_lock_args", &lock["owner_lock_args"]);
        let manager_lock_args = parse_json_str("cell.lock.manager_lock_args", &lock["manager_lock_args"]);
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": gen_das_lock_args(owner_lock_args, Some(manager_lock_args))
        });

        let type_script = cell.get("type").expect("cell.type is missing").to_owned();

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
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
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
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::AccountSaleCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);

            (capacity, lock_script, type_script, outputs_data, None)
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64 | null, // if this is null, will be calculated from sum of records.
    ///     "lock": {
    ///         "code_hash": "{{always_success}}",
    ///     },
    ///     "type": {
    ///         "code_hash": "{{income-cell-type}}"
    ///     },
    ///     "data": null | "0x...", // if this is null, will be calculated from witness.
    ///     "witness": {
    ///         "creator": null | Script, // if this is null, will be calculated from account.
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
    fn gen_income_cell(&mut self, version: u32, cell: Value) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
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
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::IncomeCellData(entity)),
                    )
                }
            }
        } else {
            let capacity = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (capacity, lock_script, type_script, outputs_data, None)
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
    ///         "account": "xxxx.bit"
    ///     }
    /// })
    /// ```
    fn gen_reverse_record_cell(&mut self, cell: Value) -> (u64, Value, Value, Vec<u8>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock = cell.get("lock").expect("cell.lock is missing");
        let owner_lock_args = parse_json_str("cell.lock.owner_lock_args", &lock["owner_lock_args"]);
        let manager_lock_args = parse_json_str("cell.lock.manager_lock_args", &lock["manager_lock_args"]);
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": gen_das_lock_args(owner_lock_args, Some(manager_lock_args))
        });

        let type_script = parse_json_script("cell.type", &cell["type"]);

        let data = &cell["data"];
        let account = parse_json_str("cell.data.account", &data["account"]);
        let outputs_data = account.as_bytes().to_vec();

        (capacity, lock_script, type_script, outputs_data)
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
    fn gen_offer_cell(&mut self, version: u32, cell: Value) -> (u64, Value, Value, Vec<u8>, Option<EntityWrapper>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock = cell.get("lock").expect("cell.lock is missing");
        let owner_lock_args = parse_json_str("cell.lock.owner_lock_args", &lock["owner_lock_args"]);
        let manager_lock_args = parse_json_str("cell.lock.manager_lock_args", &lock["manager_lock_args"]);
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": gen_das_lock_args(owner_lock_args, Some(manager_lock_args))
        });

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
                        capacity,
                        lock_script,
                        type_script,
                        outputs_data,
                        Some(EntityWrapper::OfferCellData(entity)),
                    )
                }
            }
        } else {
            let outputs_data = parse_json_hex("cell.data", &cell["data"]);
            (capacity, lock_script, type_script, outputs_data, None)
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
    ///         "code_hash": "{{balance-cell-type}}"
    ///     },
    ///     "data": null | "0x..."
    /// })
    /// ```
    fn gen_balance_cell(&mut self, cell: Value) -> (u64, Value, Value, Option<Vec<u8>>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock = cell.get("lock").expect("cell.lock is missing");
        let owner_lock_args = parse_json_str("cell.lock.owner_lock_args", &lock["owner_lock_args"]);
        let manager_lock_args = parse_json_str("cell.lock.manager_lock_args", &lock["manager_lock_args"]);
        let lock_script = json!({
          "code_hash": "{{fake-das-lock}}",
          "args": gen_das_lock_args(owner_lock_args, Some(manager_lock_args))
        });

        let type_script = cell.get("type").expect("cell.type is missing").to_owned();

        let outputs_data_opt = if !cell["data"].is_null() {
            Some(parse_json_hex("cell.data", &cell["data"]))
        } else {
            None
        };

        (capacity, lock_script, type_script, outputs_data_opt)
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
    fn gen_custom_cell(&mut self, cell: Value) -> (u64, Value, Value, Option<Vec<u8>>) {
        let capacity: u64 = parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = if !cell["type"].is_null() {
            parse_json_script("cell.type", &cell["type"])
        } else {
            Value::Null
        };
        let outputs_data_opt = if !cell["data"].is_null() {
            Some(parse_json_hex("cell.data", &cell["data"]))
        } else {
            None
        };

        (capacity, lock_script, type_script, outputs_data_opt)
    }

    // ======

    pub fn as_json(&self) -> serde_json::Value {
        let witnesses = [self.inner_witnesses.clone(), self.outer_witnesses.clone()].concat();
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
