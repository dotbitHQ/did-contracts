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

test_with_template!(test_init_account_chain, "account_init_account_chain.json");

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
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        true,
        10_400_000_000,
        Source::Input,
    );
    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Input);

    template.push_ref_cell(
        "0x0000000000000000000000000000000000002222",
        account,
        true,
        10_400_000_000,
        Source::Output,
    );
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        None,
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

test_with_template!(test_transfer_account, "account_transfer.json");

// #[test]
fn gen_edit_manager_test_data() {
    println!("====== Print edit_manager test data ======");

    let mut template = TemplateGenerator::new("edit_manager", None);
    let timestamp = 1611200000u64;

    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        true,
        10_400_000_000,
        Source::Input,
    );
    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Input);
    template.push_ref_cell(
        "0x0000000000000000000000000000000000002222",
        account,
        false,
        10_400_000_000,
        Source::Input,
    );

    template.push_ref_cell(
        "0x0000000000000000000000000000000000001111",
        account,
        true,
        10_400_000_000,
        Source::Output,
    );
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000003333",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Output);
    template.push_ref_cell(
        "0x0000000000000000000000000000000000003333",
        account,
        false,
        10_400_000_000,
        Source::Output,
    );

    template.push_witness(
        DataType::AccountCellData,
        Some((1, 1, new_entity)),
        Some((1, 1, old_entity)),
        None,
    );

    template.pretty_print();
}

test_with_template!(test_edit_manager, "account_edit_manager.json");

// #[test]
fn gen_edit_records_test_data() {
    println!("====== Print edit_records test data ======");

    let mut template = TemplateGenerator::new("edit_records", None);
    let timestamp = 1611200000u64;

    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    template.push_ref_cell(
        "0x0000000000000000000000000000000000002222",
        account,
        false,
        10_400_000_000,
        Source::Input,
    );
    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Input);

    let records = gen_account_records();

    template.push_ref_cell(
        "0x0000000000000000000000000000000000002222",
        account,
        false,
        10_400_000_000,
        Source::Output,
    );
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        Some(records),
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

test_with_template!(test_edit_records, "account_edit_records.json");

#[test]
fn gen_renew_account_test_data() {
    println!("====== Print renew_account test data ======");

    let mut template = TemplateGenerator::new("renew_account", None);
    let timestamp = 1611200000u64;

    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);
    template.push_quote_cell(1000, 200_000_000_000, Source::CellDep);
    template.push_config_cell(
        ConfigID::ConfigCellRegister,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Input);
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at + 86400 * 365,
        None,
    );
    template.push_account_cell(cell_data, None, 19_400_000_000, Source::Output);
    template.push_witness(
        DataType::AccountCellData,
        Some((1, 0, new_entity)),
        Some((1, 0, old_entity)),
        None,
    );

    template.push_signall_cell(
        "0x0300000000000000000000000000000000000000",
        500_000_000_000,
        Source::Output,
    );

    template.pretty_print();
}

test_with_template!(test_renew_account, "account_renew_account.json");

// #[test]
fn gen_recycle_expired_account_by_keeper_test_data() {
    println!("====== Print recycle_expired_account_by_keeper test data ======");

    let mut template = TemplateGenerator::new("recycle_expired_account_by_keeper", None);
    let timestamp = 1611200000u64;

    template.push_time_cell(1, timestamp, 200_000_000_000, Source::CellDep);

    template.push_config_cell(
        ConfigID::ConfigCellMain,
        true,
        100_000_000_000,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let registered_at = timestamp - 86400 * (365 + 30); // Register at 1 year and 1 month before
    let expired_at = timestamp - 86400 * 30 - 1; // Expired at 1 month + 1 second before
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        next.clone(),
        registered_at,
        expired_at,
        None,
    );
    template.push_account_cell(cell_data, None, 15_800_000_000, Source::Input);
    template.push_wallet_cell(account, 8_400_000_000 + 2_000_000_000, Source::Input);
    template.push_wallet_cell("das.bit", 8_400_000_000, Source::Input);

    template.push_wallet_cell("das.bit", 16_800_000_000, Source::Output);

    template.push_witness(
        DataType::AccountCellData,
        None,
        Some((1, 0, old_entity)),
        None,
    );

    template.pretty_print();
}

test_with_template!(
    test_recycle_expired_account_by_keeper,
    "account_recycle_expired_account_by_keeper.json"
);
