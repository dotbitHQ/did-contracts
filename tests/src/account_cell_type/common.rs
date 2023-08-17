use das_types::constants::*;
use das_types::packed::*;

use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::{self};

pub fn init(action: &str, params_opt: Option<&str>) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(util::hex_to_bytes(raw))));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-das-lock", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("eip712-lib", ContractType::Contract);
    template.push_contract_cell("account-cell-type", ContractType::Contract);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_contract_cell("ckb_multi_sign.so", ContractType::SharedLib);

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
