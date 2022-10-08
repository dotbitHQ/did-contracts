use das_types_std::constants::*;

use crate::util::constants::*;
use crate::util::template_generator::*;

pub fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("apply-register-cell-type", ContractType::Contract);

    template.push_oracle_cell(1, OracleCellType::Height, HEIGHT);
    template.push_oracle_cell(1, OracleCellType::Time, TIMESTAMP);

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);

    template
}
