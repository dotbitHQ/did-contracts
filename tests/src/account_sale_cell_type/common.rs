use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};
use serde_json::{json, Value};

pub const ACCOUNT: &str = "xxxxx.bit";
pub const SELLER: &str = "0x050000000000000000000000000000000000001111";
pub const BUYER: &str = "0x050000000000000000000000000000000000002222";
pub const PRICE: u64 = 200_000_000_000;
pub const TIMESTAMP: u64 = 1611200090u64;

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("balance-cell-type", false);
    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("account-sale-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSecondaryMarket, true, 0, Source::CellDep);

    template
}

pub fn init_with_profit_rate(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);
    template.push_contract_cell("income-cell-type", false);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);

    template
}

pub fn push_input_account_sale_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY,
        "lock": {
            "owner_lock_args": SELLER,
            "manager_lock_args": SELLER
        },
        "type": {
            "code_hash": "{{account-sale-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT,
            "price": PRICE,
            "description": "This is some account description.",
            "started_at": TIMESTAMP,
            "buyer_inviter_profit_rate": SALE_BUYER_INVITER_PROFIT_RATE
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_account_sale_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY,
        "lock": {
            "owner_lock_args": SELLER,
            "manager_lock_args": SELLER
        },
        "type": {
            "code_hash": "{{account-sale-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT,
            "price": PRICE,
            "description": "This is some account description.",
            "started_at": TIMESTAMP,
            "buyer_inviter_profit_rate": SALE_BUYER_INVITER_PROFIT_RATE
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, Some(2));
}

pub fn push_input_account_sale_cell_v1(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY,
        "lock": {
            "owner_lock_args": SELLER,
            "manager_lock_args": SELLER
        },
        "type": {
            "code_hash": "{{account-sale-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT,
            "price": PRICE,
            "description": "This is some account description.",
            "started_at": TIMESTAMP,
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(1));
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

pub fn push_output_account_sale_cell_v1(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY,
        "lock": {
            "owner_lock_args": SELLER,
            "manager_lock_args": SELLER
        },
        "type": {
            "code_hash": "{{account-sale-cell-type}}"
        },
        "witness": {
            "account": ACCOUNT,
            "price": PRICE,
            "description": "This is some account description.",
            "started_at": TIMESTAMP,
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_output(cell, Some(1));
}
