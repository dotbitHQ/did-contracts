use super::util::{constants::MAX_CYCLES, template_parser::TemplateParser};
use ckb_testtool::context::Context;

#[test]
fn test_always_success() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "../templates/always_success.json"
    );

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_always_success: {} cycles", cycles);
}
