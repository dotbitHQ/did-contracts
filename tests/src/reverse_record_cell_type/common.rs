use das_types_std::constants::*;
use das_types_std::packed::*;
use serde_json::json;

use crate::util::template_generator::*;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(Bytes::from(vec![0])));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("eip712-lib", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("reverse-record-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellReverseResolution, Source::CellDep);

    template
}

pub fn push_input_reverse_record_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str, account: &str) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{reverse-record-cell-type}}"
            },
            "data": {
                "account": account
            }
        }),
        None,
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}
