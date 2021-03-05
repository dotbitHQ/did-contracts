use super::util::{constants::*, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::bytes;
use das_types::constants::ConfigID;

// #[test]
fn gen_wallet_create_test_data() {
    println!("====== Print wallet_create transaction data ======");

    let mut template = TemplateGenerator::new("create_wallet", None);

    let source = Source::Output;
    template.push_wallet_cell("das00001.bit", 94, source);
    template.push_wallet_cell("das00002.bit", 94, source);
    template.push_wallet_cell("das00003.bit", 94, source);

    template.pretty_print();
}

#[test]
fn test_wallet_create() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "wallet_create.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_propose: {} cycles", cycles);
}

// #[test]
fn gen_wallet_withdraw_test_data() {
    println!("====== Print wallet_withdraw transaction data ======");

    let mut template = TemplateGenerator::new("withdraw_from_wallet", None);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";

    // Generate RefCells ...
    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        194,
        Source::Input,
    );
    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        194,
        Source::Output,
    );

    // Generate AccountCells ...
    let splited_account = account[..&account.len() - 4]
        .split("")
        .filter(|item| !item.is_empty())
        .collect::<Vec<&str>>();
    let account_chars = gen_account_chars(splited_account);
    let (cell_data, entity) = template.gen_account_cell_data(
        &account_chars,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        bytes::Bytes::from(account_to_id_bytes("das00014.bit")),
        1611200000u64,
        1611200000u64 + 31536000,
    );
    template.push_account_cell(
        cell_data.clone(),
        Some((1, 1, entity.clone())),
        194,
        Source::Input,
    );
    template.push_account_cell(cell_data, Some((1, 1, entity)), 194, Source::Output);

    // Generate WalletCells ...
    template.push_wallet_cell(account, 10094, Source::Input);
    template.push_wallet_cell(account, 5094, Source::Output);

    template.pretty_print();
}

// #[test]
fn test_wallet_withdraw() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "wallet_withdraw.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_propose: {} cycles", cycles);
}

// #[test]
fn gen_wallet_recycle_test_data() {
    println!("====== Print wallet_recycle transaction data ======");

    let mut template = TemplateGenerator::new("recycle_wallet", None);

    let source = Source::Output;
    template.push_wallet_cell("das00001.bit", 94, source);
    template.push_wallet_cell("das00002.bit", 94, source);
    template.push_wallet_cell("das00003.bit", 94, source);

    template.pretty_print();
}

// #[test]
fn test_wallet_recycle() {
    let mut context;
    let mut parser;
    load_template!(&mut context, &mut parser, "wallet_recycle.json");

    // build transaction
    let tx = parser.build_tx();

    // run in vm
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");

    println!("test_propose: {} cycles", cycles);
}
