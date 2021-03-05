use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;

// #[test]
fn gen_init_account_chain_test_data() {
    println!("====== Print init_account_chain test data ======");

    let mut template = TemplateGenerator::new("init_account_chain", None);

    let (cell_data, entity) = template.gen_root_account_cell_data();
    template.push_account_cell(
        cell_data,
        Some((1, 0, entity)),
        28_800_000_000,
        Source::Output,
    );

    template.pretty_print();
}

#[test]
fn test_init_account_chain() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "../templates/account_init_account_chain.json"
    );

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_always_success: {} cycles", cycles);
}
