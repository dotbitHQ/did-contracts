use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use das_types::packed::ConfigCellData;

// #[test]
fn gen_config_cell_create() {
    println!("====== Print config cell creation transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    let (cell_data, entity) = template.gen_config_cell_data();
    template.push_config_cell(cell_data, Some((1, 0, entity)), 1000, Source::Output);

    template.pretty_print();
}

#[test]
fn test_config_cell_create() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "config_cell_create.json");

    // parser
    //     .sign_by_key("0x3500349eec0f58fe28e204e4f5ce4ef93643da7c071a46a9c618632c93767ded")
    //     .unwrap();

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_config_cell_create: {} cycles", cycles);
}

// #[test]
fn gen_config_cell_edit() {
    println!("====== Print config cell editing transaction ======");

    let mut template = TemplateGenerator::new("config", None);

    let (cell_data, entity) = template.gen_config_cell_data();
    template.push_config_cell(
        cell_data.clone(),
        None::<(u32, u32, ConfigCellData)>,
        1000,
        Source::Input,
    );
    template.push_config_cell(
        cell_data,
        None::<(u32, u32, ConfigCellData)>,
        1000,
        Source::Output,
    );
    template.push_witness(
        DataType::ConfigCellData,
        Some((1, 0, entity.clone())),
        Some((1, 0, entity)), // Because there is no needs in testing, we use the same entity.
        None,
    );

    template.pretty_print();
}

#[test]
fn test_config_cell_edit() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "config_cell_edit.json");

    // parser
    //     .sign_by_key("0x3500349eec0f58fe28e204e4f5ce4ef93643da7c071a46a9c618632c93767ded")
    //     .unwrap();

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_config_cell_edit: {} cycles", cycles);
}
