use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_tool::ckb_hash::blake2b_256;
use das_types::{constants::*, packed::*, prelude::*, util as das_util};
use serde_json::{json, Value};
use std::convert::TryFrom;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("test-env", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);

    template
}

fn gen_config_cell_account() -> (Value, Value, Bytes, ConfigCellAccount) {
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
    let config_id_hex = util::hex_string(&(DataType::ConfigCellAccount as u32).to_le_bytes());
    let lock_script = json!({
      "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
      "args": CONFIG_LOCK_ARGS
    });
    let type_script = json!({
      "code_hash": "{{config-cell-type}}",
      "args": format!("0x{}", config_id_hex),
    });

    (lock_script, type_script, cell_data, entity)
}

#[test]
fn test_parse_witness_entity_config() {
    let mut template = init("test_parse_witness_entity_config");

    let (lock_script, type_script, cell_data, entity) = gen_config_cell_account();
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    let witness = das_util::wrap_entity_witness(DataType::ConfigCellAccount, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_normal_cell(&mut template, 0, CONFIG_LOCK_ARGS);
    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_parse_witness_raw_config() {
    let mut template = init("test_parse_witness_raw_config");

    // Load record_key_namespace.txt
    let mut record_key_namespace = Vec::new();
    let lines =
        util::read_lines("record_key_namespace.txt").expect("Expect file ./tests/data/record_key_namespace.txt exist.");
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
    let config_id_hex = util::hex_string(&(DataType::ConfigCellRecordKeyNamespace as u32).to_le_bytes());
    let lock_script = json!({
      "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
      "args": CONFIG_LOCK_ARGS
    });
    let type_script = json!({
      "code_hash": "{{config-cell-type}}",
      "args": format!("0x{}", config_id_hex),
    });
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    let witness = das_util::wrap_raw_witness(DataType::ConfigCellRecordKeyNamespace, raw);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn challenge_parse_witness_entity_config_data_type() {
    let mut template = init("test_parse_witness_entity_config");

    let (lock_script, type_script, cell_data, entity) = gen_config_cell_account();
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    // Simulate put the witness of the ConfigCell with wrong data type.
    let witness = das_util::wrap_entity_witness(DataType::ConfigCellProposal, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), Error::ConfigIsPartialMissing);
}

#[test]
fn challenge_parse_witness_entity_config_entity_hash() {
    let mut template = init("test_parse_witness_entity_config");

    let (lock_script, type_script, _, entity) = gen_config_cell_account();
    // Simulate put the witness of the ConfigCell with wrong hash.
    let fake_cell_data = Bytes::from(vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ]);
    template.push_cell(0, lock_script, type_script, Some(fake_cell_data), Source::CellDep);

    let witness = das_util::wrap_entity_witness(DataType::ConfigCellAccount, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), Error::ConfigCellWitnessIsCorrupted);
}

fn gen_account_cell() -> (Value, Value, Bytes, AccountCellData) {
    let entity = AccountCellData::new_builder()
        .id(AccountId::try_from(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap())
        .build();
    let cell_data = Bytes::from(blake2b_256(entity.as_slice()).to_vec());
    let lock_script = json!({
      "code_hash": "{{fake-das-lock}}",
      "args": gen_das_lock_args("0x000000000000000000000000000000000000001111", None)
    });
    let type_script = json!({
      "code_hash": "{{account-cell-type}}"
    });

    (lock_script, type_script, cell_data, entity)
}

#[test]
fn test_parse_witness_cells() {
    let mut template = init("test_parse_witness_cells");

    let index = template.cell_deps.len() as u32;
    let (lock_script, type_script, cell_data, entity) = gen_account_cell();
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    let witness = das_util::wrap_data_witness_v2(DataType::AccountCellData, 2, index, entity, Source::CellDep);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn challenge_parse_witness_cells_data_type() {
    let mut template = init("test_parse_witness_cells");

    let index = template.cell_deps.len() as u32;
    let (lock_script, type_script, cell_data, entity) = gen_account_cell();
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    // Simulate put the witness of the ConfigCell with wrong data type.
    let witness = das_util::wrap_data_witness_v2(DataType::IncomeCellData, 2, index, entity, Source::CellDep);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), Error::WitnessDataHashOrTypeMissMatch);
}

#[test]
fn challenge_parse_witness_cells_hash() {
    let mut template = init("test_parse_witness_cells");

    let index = template.cell_deps.len() as u32;
    let (lock_script, type_script, _, entity) = gen_account_cell();
    // Simulate put the witness of the ConfigCell with wrong hash.
    let fake_cell_data = Bytes::from(vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ]);
    template.push_cell(0, lock_script, type_script, Some(fake_cell_data), Source::CellDep);

    let witness = das_util::wrap_data_witness_v2(DataType::AccountCellData, 2, index, entity, Source::CellDep);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), Error::WitnessDataHashOrTypeMissMatch);
}
