use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

fn gen_cell_deps(template: &mut TemplateGenerator, height: u64, timestamp: u64) {
    template.push_height_cell(1, height, 200_000_000_000, Source::CellDep);
    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_quote_cell(1000, 500_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::CellDep,
    );
    template.push_config_cell(
        ConfigID::ConfigCellBloomFilter,
        true,
        100_000_000_000,
        Source::CellDep,
    );
}

// #[test]
fn gen_pre_register_test_data() {
    println!("====== Print pre_register test data ======");

    let mut template = TemplateGenerator::new("pre_register", None);
    let timestamp = 1611200060u64;
    let height = 1000u64;

    gen_cell_deps(&mut template, height, timestamp);

    let account = "das00001.bit";
    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        account,
        height - 4,
        100_000_000_000,
        Source::Input,
    );

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x0000000000000000000000000000000000002222",
        "0x000000000000000000000000000000000000FFFF",
        "inviter_01.bit",
        "channel_01.bit",
        1000,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        536_600_000_000,
        Source::Output,
    );

    template.pretty_print();
}

#[test]
fn test_pre_register() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "pre_register.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_pre_register: {} cycles", cycles);
}

// #[test]
fn gen_reserved_account_verification_test_data() {
    println!("====== Print pre_register test data ======");

    let mut template = TemplateGenerator::new("pre_register", None);
    let timestamp = 1611200060u64;
    let height = 1000u64;

    gen_cell_deps(&mut template, height, timestamp);

    let account = "microsoft.bit";
    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        account,
        timestamp - 60,
        100_000_000_000,
        Source::Input,
    );

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        account,
        "0x0000000000000000000000000000000000002222",
        "0x000000000000000000000000000000000000FFFF",
        "inviter_01.bit",
        "channel_01.bit",
        1000,
        timestamp,
    );
    template.push_pre_account_cell(
        cell_data,
        Some((1, 0, entity)),
        528_800_000_000,
        Source::Output,
    );

    template.pretty_print();
}

// #[test]
#[should_panic]
fn test_reserved_account_verification() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "pre_register_reserved_account.json"
    );

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_pre_register: {} cycles", cycles);
}
