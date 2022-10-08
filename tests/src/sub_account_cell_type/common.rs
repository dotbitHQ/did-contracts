use das_types_std::constants::*;
use das_types_std::packed::*;

use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("sub-account-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

pub fn init_create(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("income-cell-type", ContractType::Contract);

    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellIncome, Source::CellDep);

    template
}

pub fn init_edit(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = init(action, params_opt);

    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("ckb_sign.so", ContractType::SharedLib);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template
}
