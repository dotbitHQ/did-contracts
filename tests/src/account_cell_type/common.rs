use das_types::constants::*;
use serde_json::{json, Value};

use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| util::hex_to_bytes(raw)));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("eip712-lib", ContractType::Contract);
    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("ckb_multi_sign.so", ContractType::SharedLib);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);

    template
}

pub fn init_for_renew(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("income-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Quote, 1000);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);

    template
}

pub fn init_for_sub_account(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("income-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("sub-account-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccountBetaList, Source::CellDep);

    template
}

pub fn push_input_account_cell_v4(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": OWNER,
            "manager_lock_args": MANAGER,
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": ACCOUNT_1,
            "next": "yyyyy.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": ACCOUNT_1,
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8),
            "enable_sub_account": 0,
            "renew_sub_account_price": 0,
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None, Some(4));
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}
