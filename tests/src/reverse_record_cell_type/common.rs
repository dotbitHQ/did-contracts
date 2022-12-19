use das_types_std::constants::*;
use das_types_std::packed::*;
use serde_json::json;

use crate::util::template_generator::*;
use crate::util::{self};

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

pub fn push_dep_account_cell(template: &mut TemplateGenerator, account: &str) {
    template.push_dep(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000009999",
                "manager_lock_args": "0x050000000000000000000000000000000000009999"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": account,
                "next": "yyyyy.bit",
                "expired_at": 0,
            },
            "witness": {
                "account": account,
                "registered_at": 0,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
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

pub fn push_output_reverse_record_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str, account: &str) {
    template.push_output(
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
    );
}
