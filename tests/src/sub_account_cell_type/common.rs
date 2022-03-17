use crate::util::{self, constants::*, template_generator::*};
use das_types_std::{constants::*, packed::*};

pub const TIMESTAMP: u64 = 1611200090u64;

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("sub-account-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

pub fn init_create(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("account-cell-type", false);
    template.push_contract_cell("income-cell-type", false);

    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);

    template
}

pub fn init_edit(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("account-cell-type", false);

    template.push_shared_lib_cell("eth_sign.so", false);
    template.push_shared_lib_cell("ckb_sign.so", false);
    template.push_shared_lib_cell("secp256k1_data", true);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template
}
