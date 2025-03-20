use das_types::constants::{DataType, Source};
use serde_json::json;

use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always-success", ContractType::Contract);
    template.push_contract_cell("playground", ContractType::Contract);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    // template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    // template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);

    template.push_config_cell(DataType::ConfigCellAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellApply, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEmoji, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetEn, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetVi, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetTr, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRelease, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellUnAvailableAccount, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    template.push_header_deps(json!({
        "height": HEIGHT,
        "timestamp": TIMESTAMP,
    }));

    template
}

#[test]
fn xxx_playground() {
    let mut template = init("playground");

    push_input_playground_cell(&mut template);

    test_tx(template.as_json());
}
