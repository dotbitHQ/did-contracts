//! Here is the tests for always_success contract AND template_parser
use crate::constants::MAX_CYCLES;
use crate::template_parser::TemplateParser;
use crate::util::{deploy_contract, mock_cell, mock_input, mock_output, mock_script};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, prelude::*};

#[test]
fn should_always_success() {
    let mut context = Context::default();
    let mut parser = TemplateParser::new(
        &mut context,
        include_str!("../templates/always_success.json"),
    )
    .expect("Init template parser failed.");

    // parse transaction template
    parser.parse();
    parser.set_outputs_data(1, Bytes::from("hello world".as_bytes()));

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("always_success: {} cycles", cycles);
}
