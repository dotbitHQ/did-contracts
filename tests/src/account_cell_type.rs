use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::bytes;
use das_types::constants::{ConfigID, DataType};

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

// #[test]
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

// #[test]
fn gen_transfer_account_test_data() {
    println!("====== Print transfer_account test data ======");

    let mut template = TemplateGenerator::new("transfer_account", None);
    let timestamp = 1611200000u64;

    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let account_chars = gen_account_chars("das00001".split("").collect::<Vec<&str>>());
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        19_400_000_000,
        Source::Input,
    );
    let (cell_data, old_entity) = template.gen_account_cell_data(
        &account_chars,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        next.clone(),
        registered_at,
        expired_at,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Input);

    template.push_ref_cell(
        "0x0000000000000000000000000000000000002222",
        account,
        19_400_000_000,
        Source::Output,
    );
    let (cell_data, new_entity) = template.gen_account_cell_data(
        &account_chars,
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Output);

    template.push_witness(
        DataType::AccountCellData,
        Some((1, 1, new_entity)),
        Some((1, 1, old_entity)),
        None,
    );

    template.pretty_print();
}

// #[test]
fn test_transfer_account() {
    let mut context;
    let mut parser;
    load_template!(
        &mut context,
        &mut parser,
        "../templates/account_transfer.json"
    );

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_always_success: {} cycles", cycles);
}
