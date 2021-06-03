use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

fn init() -> TemplateGenerator {
    let mut template = TemplateGenerator::new("config", None);

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("config-cell-type", false);

    template
}

#[test]
fn gen_config_create() {
    let mut template = init();

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        0,
        Source::Input,
    );

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

    template.pretty_print();
}

test_with_template!(test_config_create, "config_create.json");

#[test]
fn gen_config_edit() {
    let mut template = init();

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        0,
        Source::Input,
    );

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

    template.pretty_print();
}

test_with_template!(test_config_edit, "config_edit.json");
