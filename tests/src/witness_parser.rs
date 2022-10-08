use crate::util::{
    self, accounts::*, constants::*, error::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_hash::blake2b_256;
use das_types_std::{constants::*, packed::*, prelude::*, util as das_util, util::EntityWrapper};
use serde_json::{json, Value};
use std::convert::TryFrom;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

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
    let config_id_hex = hex::encode(&(DataType::ConfigCellAccount as u32).to_le_bytes());
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
fn parse_witness_entity_config() {
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
fn parse_witness_raw_config() {
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
    let config_id_hex = hex::encode(&(DataType::ConfigCellRecordKeyNamespace as u32).to_le_bytes());
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
fn parse_witness_error_entity_config_data_type() {
    let mut template = init("test_parse_witness_entity_config");

    let (lock_script, type_script, cell_data, entity) = gen_config_cell_account();
    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    // Simulate put the witness of the ConfigCell with wrong data type.
    let witness = das_util::wrap_entity_witness(DataType::ConfigCellProposal, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::ConfigIsPartialMissing);
}

#[test]
fn parse_witness_error_entity_config_entity_hash() {
    let mut template = init("test_parse_witness_entity_config");

    let (lock_script, type_script, _, entity) = gen_config_cell_account();
    // Simulate put the witness of the ConfigCell with wrong hash.
    let fake_cell_data = Bytes::from(vec![0; 32]);
    template.push_cell(0, lock_script, type_script, Some(fake_cell_data), Source::CellDep);

    let witness = das_util::wrap_entity_witness(DataType::ConfigCellAccount, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness.raw_data()));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::ConfigCellWitnessIsCorrupted);
}

fn gen_account_cell(outputs_data_opt: Option<String>) -> (Value, EntityWrapper) {
    let entity = AccountCellData::new_builder()
        .id(AccountId::try_from(vec![0; 20]).unwrap())
        .build();

    let lock = parse_json_script_das_lock(
        "",
        &json!({
            "owner_lock_args": OWNER,
            "manager_lock_args": MANAGER
        }),
    );
    // The outputs_data of AccountCell is simplified for witnesses tests, so it IS invalid in other tests.
    let outputs_data = if let Some(val) = outputs_data_opt {
        val
    } else {
        util::bytes_to_hex(&blake2b_256(entity.as_slice()))
    };
    let cell = json!({
        "tmp_type": "full",
        "capacity": util::gen_account_cell_capacity(5),
        "lock": lock,
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "tmp_data": outputs_data
    });

    (cell, EntityWrapper::AccountCellData(entity))
}

#[test]
fn parse_witness_cells() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::AccountCellData, 3, Some(entity));

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn parse_witness_end_with_unknown_witnesses() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::AccountCellData, 3, Some(entity));
    template
        .outer_witnesses
        .push(String::from("0x11112222333344445555666677778888"));
    template
        .outer_witnesses
        .push(String::from("0x11112222333344445555666677778888"));

    push_input_test_env_cell(&mut template);
    test_tx(template.as_json());
}

#[test]
fn parse_witness_error_cells_data_type() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(None);
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::IncomeCellData, 3, Some(entity));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::WitnessDataHashOrTypeMissMatch);
}

#[test]
fn parse_witness_error_cells_hash() {
    let mut template = init("test_parse_witness_cells");

    let (cell, entity) = gen_account_cell(Some(String::from(
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    )));
    template.push_cell_json_with_entity(cell, Source::CellDep, DataType::IncomeCellData, 3, Some(entity));

    push_input_test_env_cell(&mut template);
    challenge_tx(template.as_json(), ErrorCode::WitnessDataHashOrTypeMissMatch);
}
