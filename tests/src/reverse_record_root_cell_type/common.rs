use das_types::constants::*;
use serde_json::json;

use crate::util::constants::*;
use crate::util::template_generator::*;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(vec![0]));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("reverse-record-root-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellReverseResolution, Source::CellDep);

    template
}

pub fn push_input_reverse_record_root_cell(template: &mut TemplateGenerator) {
    let current_root = template.smt_with_history.current_root();
    template.push_input(
        json!({
            "header": {
                "timestamp": TIMESTAMP,
            },
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
        None,
    );
}

pub fn push_output_reverse_record_root_cell(template: &mut TemplateGenerator) {
    let current_root = template.smt_with_history.current_root();
    template.push_output(
        json!({
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
    );
}
