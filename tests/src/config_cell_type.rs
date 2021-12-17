use super::util::{constants::*, template_common_cell::*, template_generator::*, template_parser::*};
use das_types::constants::*;

fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("config", None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("config-cell-type", false);

    template
}

#[test]
fn test_config_create() {
    let mut template = init();

    push_input_normal_cell(&mut template, 0, CONFIG_LOCK_ARGS);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellProposal, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::Output);
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

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellProposal, true, 0, Source::Input);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::Input);

    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellApply, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellCharSetDigit, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellIncome, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellProposal, true, 0, Source::Output);
    template.push_config_cell(DataType::ConfigCellProfitRate, true, 0, Source::Output);

    test_tx(template.as_json());
}
