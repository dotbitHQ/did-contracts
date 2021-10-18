use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};

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
