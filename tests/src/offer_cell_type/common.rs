use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};
use serde_json::{json, Value};

pub const ACCOUNT: &str = "xxxxx.bit";
pub const SELLER: &str = "0x050000000000000000000000000000000000001111";
pub const BUYER: &str = "0x050000000000000000000000000000000000002222";
pub const PRICE: u64 = 200_000_000_000;
pub const TIMESTAMP: u64 = 1611200090u64;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, Some(Bytes::from(vec![0])));

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("balance-cell-type", false);
    template.push_contract_cell("offer-cell-type", false);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSecondaryMarket, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, true, 0, Source::CellDep);

    template
}

pub fn init_with_timestamp(action: &str) -> TemplateGenerator {
    let mut template = init(action);

    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("income-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

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
            "account": ACCOUNT,
            "price": "200_000_000_000",
            "message": "Take my money.🍀",
            "inviter_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
            },
            "channel_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
            }
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None);
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
            "account": ACCOUNT,
            "price": "200_000_000_000",
            "message": "Take my money.🍀",
            "inviter_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
            },
            "channel_lock": {
                "code_hash": "{{fake-das-lock}}",
                "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
            }
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, None);
}
