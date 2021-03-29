use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;

// #[test]
fn gen_apply_register_test_data() {
    println!("====== Print apply_register test data ======");

    let mut template = TemplateGenerator::new("apply_register", None);

    let height = 1000u64;
    template.push_height_cell(1, height, 1000, Source::CellDep);

    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        "das00001.bit",
        height,
        1000,
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
