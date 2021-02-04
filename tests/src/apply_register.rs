use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

// #[test]
fn gen_transaction_data() {
    println!("====== Print generated transaction data ======");

    let mut template = TemplateGenerator::new("apply_register", None);

    let timestamp = 1611200000;
    template.gen_time_cell(1, timestamp);

    let dep_entity = template.gen_config_cell(Source::CellDep);
    template.gen_witness(
        DataType::ConfigCellData,
        None,
        None,
        Some((1, 4, dep_entity)),
    );

    let account = "✨dasdas✨";
    let account_chars = gen_account_chars(account.split("").collect());
    template.gen_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        &account_chars,
        timestamp,
        Source::Output,
    );

    template.pretty_print();
}

#[test]
fn test_apply_register() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "apply_register.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_apply_register: {} cycles", cycles);
}
