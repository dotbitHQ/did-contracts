use super::util::{constants::*, template_common_cell::*, template_generator::*, template_parser::*};
use das_types_std::constants::*;

fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("config", None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("config-cell-type", ContractType::Contract);

    template
}

#[test]
fn test_config_create() {
    let mut template = init();

    push_input_normal_cell(&mut template, 0, CONFIG_LOCK_ARGS);

    template.push_config_cell(DataType::ConfigCellAccount, Source::Output);
    template.push_config_cell(DataType::ConfigCellApply, Source::Output);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::Output);
    template.push_config_cell(DataType::ConfigCellIncome, Source::Output);
    template.push_config_cell(DataType::ConfigCellMain, Source::Output);
    template.push_config_cell(DataType::ConfigCellPrice, Source::Output);
    template.push_config_cell(DataType::ConfigCellProposal, Source::Output);
    template.push_config_cell(DataType::ConfigCellProfitRate, Source::Output);
    // template.push_config_cell(
    //     DataType::ConfigCellPreservedAccount00,
    //     true,
    //     0,
    //     Source::Output,
    // );

    test_tx(template.as_json());
}

#[test]
fn test_config_edit() {
    let mut template = init();

    push_input_normal_cell(&mut template, 0, CONFIG_LOCK_ARGS);

    template.push_config_cell(DataType::ConfigCellAccount, Source::Input);
    template.push_config_cell(DataType::ConfigCellApply, Source::Input);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::Input);
    template.push_config_cell(DataType::ConfigCellIncome, Source::Input);
    template.push_config_cell(DataType::ConfigCellMain, Source::Input);
    template.push_config_cell(DataType::ConfigCellPrice, Source::Input);
    template.push_config_cell(DataType::ConfigCellProposal, Source::Input);
    template.push_config_cell(DataType::ConfigCellProfitRate, Source::Input);

    template.push_config_cell(DataType::ConfigCellAccount, Source::Output);
    template.push_config_cell(DataType::ConfigCellApply, Source::Output);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, Source::Output);
    template.push_config_cell(DataType::ConfigCellIncome, Source::Output);
    template.push_config_cell(DataType::ConfigCellMain, Source::Output);
    template.push_config_cell(DataType::ConfigCellPrice, Source::Output);
    template.push_config_cell(DataType::ConfigCellProposal, Source::Output);
    template.push_config_cell(DataType::ConfigCellProfitRate, Source::Output);

    test_tx(template.as_json());
}
