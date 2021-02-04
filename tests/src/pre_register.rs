use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;

// #[test]
fn gen_transaction_data() {
    println!("====== Print generated transaction data ======");

    let mut template = TemplateGenerator::new("pre_register", None);

    let timestamp = 1611200060;
    template.gen_time_cell(1, timestamp);

    let dep_entity = template.gen_config_cell(Source::CellDep);
    template.gen_witness(
        DataType::ConfigCellData,
        None,
        None,
        Some((1, 5, dep_entity)),
    );

    let account = "✨dasdas✨";
    let account_chars = gen_account_chars(account.split("").collect());
    template.gen_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        &account_chars,
        timestamp - 60,
        Source::Input,
    );

    let (new_entity, _) =
        template.gen_pre_account_cell(account, &account_chars, timestamp, Source::Output);
    template.gen_witness(
        DataType::PreAccountCellData,
        Some((1, 0, new_entity)),
        None,
        None,
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
