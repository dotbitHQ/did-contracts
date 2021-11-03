use crate::util::{self, constants::*, template_generator::*};
use das_types::{constants::DataType, packed::*};

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

pub fn init_with_timestamp(action: &str) -> (TemplateGenerator, u64) {
    let mut template = init(action);
    let timestamp = 1611200000u64;

    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("income-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::CellDep);

    (template, timestamp)
}
