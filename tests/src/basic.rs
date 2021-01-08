use super::constants::MAX_CYCLES;
use super::template_parser::TemplateParser;
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::bytes::Bytes;

#[test]
fn test_tamplate_parser_api() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "../templates/tamplate_parser_api.json"
    );

    // set output data manually
    parser.set_outputs_data(0, Bytes::from("hello world".as_bytes()));
    // eprintln!("parser.outputs_data = {:#?}", parser.outputs_data);

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_tamplate_parser_api: {} cycles", cycles);
}

#[test]
fn test_tamplate_parser_sighash_all_support() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "../templates/tamplate_parser_sighash_all_support.json"
    );

    // sign transaction
    let private_keys = vec![
        "0x3500349eec0f58fe28e204e4f5ce4ef93643da7c071a46a9c618632c93767ded",
        "0x24c22cafdc7bb24f75c6e23a9bc5772e74005a3d41f40e57c3008a609db6f76b",
    ];
    parser.sign_by_keys(private_keys).unwrap();
    // eprintln!("parser.witnesses = {:#?}", parser.witnesses);

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!(
        "test_tamplate_parser_sighash_all_support: {} cycles",
        cycles
    );
}
