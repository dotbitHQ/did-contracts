use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

// #[test]
fn gen_config_create() {
    println!("====== Print config cell creation transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    template.push_config_cell(ConfigID::ConfigCellMain, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellRegister, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellBloomFilter, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellMarket, true, 0, Source::Output);

    template.pretty_print();
}

test_with_template!(test_config_create, "config_create.json");

// #[test]
fn gen_config_edit() {
    println!("====== Print config cell editing transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    template.push_config_cell(ConfigID::ConfigCellMain, false, 0, Source::Input);
    template.push_config_cell(ConfigID::ConfigCellRegister, false, 0, Source::Input);
    template.push_config_cell(ConfigID::ConfigCellBloomFilter, false, 0, Source::Input);
    template.push_config_cell(ConfigID::ConfigCellMarket, false, 0, Source::Input);

    template.push_config_cell(ConfigID::ConfigCellMain, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellRegister, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellBloomFilter, true, 0, Source::Output);
    template.push_config_cell(ConfigID::ConfigCellMarket, true, 0, Source::Output);

    template.pretty_print();
}

test_with_template!(test_config_edit, "config_edit.json");
