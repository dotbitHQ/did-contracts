use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

// #[test]
fn gen_config_create_test_data() {
    println!("====== Print config cell creation transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellBloomFilter,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellMarket,
        true,
        100_000_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_config_create, "config_create.json");

// #[test]
fn gen_config_edit_test_data() {
    println!("====== Print config cell editing transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        false,
        100_000_000_000,
        Source::Input,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        false,
        100_000_000_000,
        Source::Input,
    );
    template.push_config_cell(
        ConfigID::ConfigCellBloomFilter,
        false,
        100_000_000_000,
        Source::Input,
    );
    template.push_config_cell(
        ConfigID::ConfigCellMarket,
        false,
        100_000_000_000,
        Source::Input,
    );
    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellBloomFilter,
        true,
        100_000_000_000,
        Source::Output,
    );
    template.push_config_cell(
        ConfigID::ConfigCellMarket,
        true,
        100_000_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_config_edit, "config_edit.json");
