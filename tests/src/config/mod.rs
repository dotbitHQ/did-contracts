use das_types::constants::*;
use serde_json::{json, Value};

use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template
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

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);

    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn test_config_records_key_namespace_loading() {
    let mut template = init("test_config_records_key_namespace_loading");

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    push_input_test_env_cell(&mut template);

    test_tx(template.as_json());
}
