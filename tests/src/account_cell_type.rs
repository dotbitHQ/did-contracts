use super::util::{
    constants::*, hex_to_bytes, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::bytes;
use das_types::{constants::DataType, packed::*};

fn init(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(
        action,
        params_opt.map(|raw| Bytes::from(hex_to_bytes(raw).unwrap())),
    );
    let timestamp = 1611200000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("account-cell-type", false);

    template.push_time_cell(1, timestamp, 0, Source::CellDep);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);

    (template, timestamp)
}

#[test]
fn gen_init_account_chain() {
    let (mut template, _) = init("init_account_chain", None);

    template.push_signall_cell(
        "0x0000000000000000000000000000000000000000",
        0,
        Source::Input,
    );

    let (cell_data, entity) = template.gen_root_account_cell_data();
    template.push_account_cell(
        "0x0000000000000000000000000000000000000000",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        cell_data,
        Some((1, 0, entity)),
        28_800_000_000,
        Source::Output,
    );

    template.write_template("account_init_account_chain.json");
}

test_with_template!(test_init_account_chain, "account_init_account_chain.json");

#[test]
fn gen_transfer_account() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        cell_data,
        None,
        19_400_000_000,
        Source::Input,
    );

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        19_400_000_000,
        Source::Output,
    );

    template.push_witness(
        DataType::AccountCellData,
        Some((1, 0, new_entity)),
        Some((1, 0, old_entity)),
        None,
    );

    template.write_template("account_transfer.json");
}

test_with_template!(test_transfer_account, "account_transfer.json");

#[test]
fn gen_edit_manager() {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        19_400_000_000,
        Source::Input,
    );

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000003333",
        cell_data,
        None,
        19_400_000_000,
        Source::Output,
    );

    template.push_witness(
        DataType::AccountCellData,
        Some((1, 0, new_entity)),
        Some((1, 0, old_entity)),
        None,
    );

    template.write_template("account_edit_manager.json");
}

test_with_template!(test_edit_manager, "account_edit_manager.json");

#[test]
fn gen_edit_records() {
    let (mut template, timestamp) = init("edit_records", Some("0x01"));

    template.push_config_cell(
        DataType::ConfigCellRecordKeyNamespace,
        true,
        0,
        Source::CellDep,
    );

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        19_400_000_000,
        Source::Input,
    );

    let records = vec![
        AccountRecordParam {
            type_: "address",
            key: "eth",
            label: "Personal",
            value: hex_to_bytes("0x00000000000000000000").unwrap(),
        },
        AccountRecordParam {
            type_: "address",
            key: "eth",
            label: "Company",
            value: hex_to_bytes("0x00000000000000000000").unwrap(),
        },
        AccountRecordParam {
            type_: "address",
            key: "btc",
            label: "Personal",
            value: hex_to_bytes("0x00000000000000000000").unwrap(),
        },
        AccountRecordParam {
            type_: "profile",
            key: "phone",
            label: "Mars",
            value: bytes::Bytes::from("120981203982901389398390".as_bytes()),
        },
        AccountRecordParam {
            type_: "profile",
            key: "email",
            label: "Company",
            value: bytes::Bytes::from("xxxxx@mars.bit".as_bytes()),
        },
        AccountRecordParam {
            type_: "custom_key",
            key: "xxxx",
            label: "xxxxxx",
            value: hex_to_bytes("0x00000000000000000000").unwrap(),
        },
    ];

    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        next.clone(),
        registered_at,
        expired_at,
        Some(gen_account_records(records)),
    );
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        19_400_000_000,
        Source::Output,
    );

    template.push_witness(
        DataType::AccountCellData,
        Some((1, 0, new_entity)),
        Some((1, 0, old_entity)),
        None,
    );

    template.write_template("account_edit_records.json");
}

test_with_template!(test_edit_records, "account_edit_records.json");

#[test]
fn gen_renew_account() {
    let (mut template, timestamp) = init("renew_account", None);

    template.push_contract_cell("income-cell-type", false);

    template.push_quote_cell(1000, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);

    let account = "das00001.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        20_000_000_000,
        Source::Input,
    );
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        next.clone(),
        registered_at,
        expired_at + 86400 * 365,
        None,
    );
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        20_000_000_000,
        Source::Output,
    );
    template.push_witness(
        DataType::AccountCellData,
        Some((1, 0, new_entity)),
        Some((1, 0, old_entity)),
        None,
    );

    let income_records = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000000",
        capacity: 20_000_000_000,
    }];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(
        cell_data,
        Some((1, 1, entity)),
        20_000_000_000,
        Source::Input,
    );

    let income_records = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000",
            capacity: 20_000_000_000,
        },
        // Profit to DAS
        IncomeRecordParam {
            belong_to: "0x0300000000000000000000000000000000000000",
            capacity: 50_000_000_000,
        },
    ];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(
        cell_data,
        Some((1, 1, entity)),
        70_000_000_000,
        Source::Output,
    );

    template.write_template("account_renew_account.json");
}

test_with_template!(test_renew_account, "account_renew_account.json");

#[test]
fn gen_recycle_expired_account_by_keeper() {
    let (mut template, timestamp) = init("recycle_expired_account_by_keeper", None);

    let account = "das00001.bit";
    let registered_at = timestamp - 86400 * (365 + 30); // Register at 1 year and 1 month before
    let expired_at = timestamp - 86400 * 30 - 1; // Expired at 1 month + 1 second before
    let next = bytes::Bytes::from(account_to_id_bytes("das00014.bit"));

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next.clone(), registered_at, expired_at, None);
    template.push_account_cell(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        Some((1, 0, old_entity)),
        21_200_000_000,
        Source::Input,
    );

    template.push_signall_cell(
        "0x0000000000000000000000000000000000001111",
        21_200_000_000,
        Source::Output,
    );

    template.write_template("account_recycle_expired_account_by_keeper.json");
}

test_with_template!(
    test_recycle_expired_account_by_keeper,
    "account_recycle_expired_account_by_keeper.json"
);
