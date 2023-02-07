use das_types_std::constants::*;
use das_types_std::packed::*;
use serde_json::json;

use crate::util::constants::REVERSE_RECORD_BASIC_CAPACITY;
use crate::util::template_generator::*;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(Bytes::from(vec![0])));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("reverse-record-root-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellReverseResolution, Source::CellDep);

    template
}

pub fn push_input_reverse_record_root_cell(template: &mut TemplateGenerator) {
    let current_root = template.smt_with_history.current_root();
    template.push_input(
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
        None,
    );
}
