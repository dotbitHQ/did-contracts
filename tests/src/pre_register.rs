use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::packed::Script;

// #[test]
fn gen_pre_register_test_data() {
    println!("====== Print pre_register test data ======");

    let mut template = TemplateGenerator::new("pre_register", None);

    let timestamp = 1611200060;
    template.push_time_cell(1, timestamp, 1000, Source::CellDep);

    template.push_quote_cell(1000, 1000, Source::CellDep);

    let (cell_data, entity) = template.gen_config_cell_data();
    template.push_config_cell(cell_data, Some((1, 6, entity)), 1000, Source::CellDep);

    let account_chars = gen_account_chars("das00001".split("").collect());
    template.push_apply_register_cell(
        "0x9af92f5e690f4669ca543deb99af8385b12624cc",
        &account_chars,
        timestamp - 60,
        1000,
        Source::Input,
    );

    let (cell_data, entity) = template.gen_pre_account_cell_data(
        &account_chars,
        "0x0000000000000000000000000000000000002222",
        "0x000000000000000000000000000000000000FFFF",
        "inviter_01.bit",
        "channel_01.bit",
        1000,
        timestamp,
    );
    template.push_pre_account_cell(cell_data, Some((1, 0, entity)), 5308, Source::Output);

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
