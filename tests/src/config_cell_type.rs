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

#[test]
fn test_config_create() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "config_create.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_config_cell_create: {} cycles", cycles);
}

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

#[test]
fn test_config_edit() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "config_edit.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_config_cell_edit: {} cycles", cycles);
}
