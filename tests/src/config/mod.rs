use ckb_hash::blake2b_256;
use das_types::constants::*;
use das_types::packed::*;
use das_types::prelude::*;
use das_types::util as das_util;
use serde_json::{json, Value};

use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template
}

fn gen_config_cell_account() -> (Vec<u8>, ConfigCellAccount) {
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
        .transfer_account_throttle(Uint32::from(DAY_SEC as u32))
        .edit_manager_throttle(Uint32::from(HOUR_SEC as u32))
        .edit_records_throttle(Uint32::from(600))
        .build();
    let cell_data = blake2b_256(entity.as_slice()).to_vec();

    (cell_data, entity)
}

fn gen_lock_script(args_opt: Option<&str>) -> Value {
    match args_opt {
        Some(args) => {
            json!({
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": args
            })
        }
        None => {
            json!({
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": CONFIG_LOCK_ARGS
            })
        }
    }
}

fn gen_type_script(data_type: DataType) -> Value {
    let config_id_hex = hex::encode(&(data_type as u32).to_le_bytes());
    json!({
      "code_hash": "{{config-cell-type}}",
      "args": format!("0x{}", config_id_hex),
    })
}

#[test]
fn test_config_account_loading() {
    let mut template = init("test_config_account_loading");

    let (cell_data, entity) = gen_config_cell_account();
    let lock_script = gen_lock_script(None);
    let type_script = gen_type_script(DataType::ConfigCellAccount);

    template.push_cell(0, lock_script, type_script, Some(cell_data), Source::CellDep);

    let witness = das_util::wrap_entity_witness_v2(DataType::ConfigCellAccount, entity);
    template.outer_witnesses.push(util::bytes_to_hex(&witness));

    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_config_records_key_namespace_loading() {
    let mut template = init("test_config_records_key_namespace_loading");

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

    let cell_data = blake2b_256(raw.as_slice()).to_vec();
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

    let witness = das_util::wrap_raw_witness_v2(DataType::ConfigCellRecordKeyNamespace, raw);
    template.outer_witnesses.push(util::bytes_to_hex(&witness));

    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}
