use das_types::constants::*;
use serde_json::{json, Value};

use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub const PRICE: u64 = 200_000_000_000;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(vec![0]));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("eip712-lib", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("offer-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSecondaryMarket, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);

    template
}

pub fn init_with_timestamp(action: &str) -> TemplateGenerator {
    let mut template = init(action);

    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("income-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, Source::CellDep);

    template
}

pub fn push_input_offer_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": 0,
        "lock": {
            "owner_lock_args": BUYER,
            "manager_lock_args": BUYER,
        },
        "type": {
            "code_hash": "{{offer-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT_1,
            "price": "200_000_000_000",
            "message": "Take my money.🍀",
            "inviter_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(INVITER, None)
            },
            "channel_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(CHANNEL, None)
            }
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None, None);
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_offer_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": 0,
        "lock": {
            "owner_lock_args": BUYER,
            "manager_lock_args": BUYER,
        },
        "type": {
            "code_hash": "{{offer-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT_1,
            "price": "200_000_000_000",
            "message": "Take my money.🍀",
            "inviter_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(INVITER, None)
            },
            "channel_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(CHANNEL, None)
            }
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, None);
}
