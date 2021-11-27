use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};
use serde_json::{json, Value};

pub fn init(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));
    let timestamp = 1611200000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("balance-cell-type", false);
    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("account-sale-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSecondaryMarket, true, 0, Source::CellDep);

    (template, timestamp)
}

pub fn init_with_profit_rate(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init(action, params_opt);
    template.push_contract_cell("income-cell-type", false);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);

    (template, timestamp)
}

pub fn push_input_account_sale_cell(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": "20_100_000_000",
        "lock": {
            "owner_lock_args": "0x050000000000000000000000000000000000001111",
            "manager_lock_args": "0x050000000000000000000000000000000000001111"
        },
        "type": {
            "code_hash": "{{account-sale-cell-type}}"
        },
        "witness": {
            "account": "xxxxx.bit",
            "price": "0",
            "description": "This is some account description.",
            "started_at": 0
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_empty_witness();
}
