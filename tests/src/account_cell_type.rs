use super::util::{constants::*, hex_to_bytes, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_core::error::Error;
use das_types::{constants::DataType, packed::*};

fn init(action: &str, params_opt: Option<&str>) -> (TemplateGenerator, u64) {
    let mut template = TemplateGenerator::new(action, params_opt.map(|raw| Bytes::from(hex_to_bytes(raw))));
    let timestamp = 1611200000u64;

    template.push_contract_cell("always_success", true);
    template.push_contract_cell("fake-das-lock", true);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", true);
    template.push_contract_cell("account-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Time, timestamp);

    template.push_config_cell(DataType::ConfigCellMain, true, 0, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellAccount, true, 0, Source::CellDep);

    (template, timestamp)
}

#[test]
fn gen_account_init_account_chain() {
    let (mut template, _) = init("init_account_chain", None);

    template.push_signall_cell("0x0000000000000000000000000000000000000000", 0, Source::Input);

    let (cell_data, entity) = template.gen_root_account_cell_data();
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000000000",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        cell_data,
        Some((1, 0, entity)),
        28_800_000_000,
        Source::Output,
    );

    template.write_template("account_init_account_chain.json");
}

test_with_template!(test_account_init_account_chain, "account_init_account_chain.json");

#[test]
fn gen_account_transfer() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY - ACCOUNT_OPERATE_FEE,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.write_template("account_transfer.json");
}

test_with_template!(test_account_transfer, "account_transfer.json");

test_with_generator!(test_account_transfer_with_eip712, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x051100000000000000000000000000000000001111",
        "0x052200000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );
    template.push_das_lock_witness("49b2584aa12dc0c6e0f05eb91c14e2f823a1bdd9a129267141ebae06c598059b");

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x050000000000000000000000000000000000002222",
        "0x050000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY - ACCOUNT_OPERATE_FEE,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_transfer_account_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) = template.gen_account_cell_data(
            account,
            next_account,
            registered_at,
            expired_at,
            timestamp - 86400 + 1,
            0,
            0,
            None,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let (cell_data, new_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000002222",
            "0x0000000000000000000000000000000000002222",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000002222",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity.clone())),
            Some((2, 0, old_entity.clone())),
            None,
        );
        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 1, new_entity)),
            Some((2, 1, old_entity)),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_transfer_invalid_manager_lock,
    Error::AccountCellManagerLockShouldBeModified,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000001111",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let (cell_data, new_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000002222",
            "0x0000000000000000000000000000000000003333",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity)),
            Some((2, 0, old_entity)),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_transfer_too_often, Error::AccountCellThrottle, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) = template.gen_account_cell_data(
        account,
        next_account,
        registered_at,
        expired_at,
        timestamp - 86400 + 1,
        0,
        0,
        None,
    );
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000001111",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, timestamp, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000002222",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.as_json()
});

#[test]
fn gen_account_edit_manager() {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, timestamp, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000003333",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.write_template("account_edit_manager.json");
}

test_with_template!(test_account_edit_manager, "account_edit_manager.json");

test_with_generator!(test_account_edit_manager_with_eip712, || {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x050000000000000000000000000000000000001111",
        "0x050000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );
    template.push_das_lock_witness("5e570d65a13b1ada6bf99b0e1f89c5eed9d05ab06826b719ffb9cdf603636cea");

    let (cell_data, new_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, timestamp, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x050000000000000000000000000000000000001111",
        "0x050000000000000000000000000000000000003333",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_edit_manager_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("edit_manager", Some("0x00"));

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let (cell_data, new_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, timestamp, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000003333",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000003333",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity.clone())),
            Some((2, 0, old_entity.clone())),
            None,
        );
        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 1, new_entity)),
            Some((2, 1, old_entity)),
            None,
        );

        template.as_json()
    }
);

#[test]
fn gen_account_edit_records() {
    let (mut template, timestamp) = init("edit_records", Some("0x01"));

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );

    let records = vec![
        AccountRecordParam {
            type_: "address",
            key: "eth",
            label: "Personal",
            value: hex_to_bytes("0x00000000000000000000"),
        },
        AccountRecordParam {
            type_: "address",
            key: "eth",
            label: "Company",
            value: hex_to_bytes("0x00000000000000000000"),
        },
        AccountRecordParam {
            type_: "address",
            key: "btc",
            label: "Personal",
            value: hex_to_bytes("0x00000000000000000000"),
        },
        AccountRecordParam {
            type_: "dweb",
            key: "ipfs",
            label: "Mars",
            value: "120981203982901389398390".as_bytes().to_vec(),
        },
        AccountRecordParam {
            type_: "profile",
            key: "email",
            label: "Company",
            value: "xxxxx@mars.bit".as_bytes().to_vec(),
        },
        AccountRecordParam {
            type_: "custom_key",
            key: "xxxx",
            label: "xxxxxx",
            value: hex_to_bytes("0x00000000000000000000"),
        },
    ];

    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        next_account,
        registered_at,
        expired_at,
        0,
        0,
        timestamp,
        Some(gen_account_records(records)),
    );
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.write_template("account_edit_records.json");
}

test_with_template!(test_account_edit_records, "account_edit_records.json");

test_with_generator!(test_account_edit_records_with_eip712, || {
    let (mut template, timestamp) = init("edit_records", Some("0x01"));

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    // inputs
    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x050000000000000000000000000000000000001111",
        "0x050000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Input,
    );
    template.push_das_lock_witness("ebab7c4e0728c679b6c7a32565843f99315d03d69d77c3d543fe7e87d8d76355");

    // outputs
    let records = vec![AccountRecordParam {
        type_: "address",
        key: "eth",
        label: "Personal",
        value: hex_to_bytes("0x00000000000000000000"),
    }];
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        next_account,
        registered_at,
        expired_at,
        0,
        0,
        timestamp,
        Some(gen_account_records(records)),
    );
    template.push_account_cell::<AccountCellData>(
        "0x050000000000000000000000000000000000001111",
        "0x050000000000000000000000000000000000002222",
        cell_data,
        None,
        1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
        Source::Output,
    );

    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_edit_records_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("edit_records", Some("0x01"));

        template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let records = vec![
            AccountRecordParam {
                type_: "address",
                key: "eth",
                label: "Personal",
                value: hex_to_bytes("0x00000000000000000000"),
            },
            AccountRecordParam {
                type_: "address",
                key: "eth",
                label: "Company",
                value: hex_to_bytes("0x00000000000000000000"),
            },
            AccountRecordParam {
                type_: "address",
                key: "btc",
                label: "Personal",
                value: hex_to_bytes("0x00000000000000000000"),
            },
            AccountRecordParam {
                type_: "dweb",
                key: "ipfs",
                label: "Mars",
                value: "120981203982901389398390".as_bytes().to_vec(),
            },
            AccountRecordParam {
                type_: "profile",
                key: "email",
                label: "Company",
                value: "xxxxx@mars.bit".as_bytes().to_vec(),
            },
            AccountRecordParam {
                type_: "custom_key",
                key: "xxxx",
                label: "xxxxxx",
                value: hex_to_bytes("0x00000000000000000000"),
            },
        ];

        let (cell_data, new_entity) = template.gen_account_cell_data(
            account,
            next_account,
            registered_at,
            expired_at,
            0,
            0,
            timestamp,
            Some(gen_account_records(records)),
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data.clone(),
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity.clone())),
            Some((2, 0, old_entity.clone())),
            None,
        );
        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 1, new_entity)),
            Some((2, 1, old_entity)),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_edit_records_invalid_char,
    Error::AccountCellRecordKeyInvalid,
    || {
        let (mut template, timestamp) = init("edit_records", Some("0x01"));

        template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let records = vec![AccountRecordParam {
            type_: "custom_key",
            key: "xxx+",
            label: "xxxxx",
            value: hex_to_bytes("0x00000000000000000000"),
        }];

        let (cell_data, new_entity) = template.gen_account_cell_data(
            account,
            next_account,
            registered_at,
            expired_at,
            0,
            0,
            timestamp,
            Some(gen_account_records(records)),
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity)),
            Some((2, 0, old_entity)),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_edit_records_invalid_key,
    Error::AccountCellRecordKeyInvalid,
    || {
        let (mut template, timestamp) = init("edit_records", Some("0x01"));

        template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Input,
        );

        let records = vec![AccountRecordParam {
            type_: "dweb",
            key: "xxxx",
            label: "xxxxx",
            value: hex_to_bytes("0x00000000000000000000"),
        }];

        let (cell_data, new_entity) = template.gen_account_cell_data(
            account,
            next_account,
            registered_at,
            expired_at,
            0,
            0,
            timestamp,
            Some(gen_account_records(records)),
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            1_200_000_000 + ACCOUNT_BASIC_CAPACITY + ACCOUNT_PREPARED_FEE_CAPACITY,
            Source::Output,
        );

        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity)),
            Some((2, 0, old_entity)),
            None,
        );

        template.as_json()
    }
);

#[test]
fn gen_account_renew() {
    let (mut template, timestamp) = init("renew_account", None);

    template.push_contract_cell("income-cell-type", false);

    template.push_oracle_cell(1, OracleCellType::Quote, 1000);
    template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400;
    let expired_at = timestamp + 31536000 - 86400;

    // inputs
    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        20_000_000_000,
        Source::Input,
    );

    let income_records = vec![IncomeRecordParam {
        belong_to: "0x0000000000000000000000000000000000000000".to_string(),
        capacity: 20_000_000_000,
    }];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(cell_data, Some((1, 1, entity)), 20_000_000_000, Source::Input);

    // outputs
    let (cell_data, new_entity) = template.gen_account_cell_data(
        account,
        next_account,
        registered_at,
        expired_at + 86400 * 365,
        0,
        0,
        0,
        None,
    );
    template.push_account_cell::<AccountCellData>(
        "0x0000000000000000000000000000000000001111",
        "0x0000000000000000000000000000000000002222",
        cell_data,
        None,
        20_000_000_000,
        Source::Output,
    );
    template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
        DataType::AccountCellData,
        Some((2, 0, new_entity)),
        Some((2, 0, old_entity)),
        None,
    );

    let income_records = vec![
        IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        },
        // Profit to DAS
        IncomeRecordParam {
            belong_to: "0x0300000000000000000000000000000000000000".to_string(),
            capacity: 500_000_000_000,
        },
    ];
    let (cell_data, entity) =
        template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
    template.push_income_cell(cell_data, Some((1, 1, entity)), 520_000_000_000, Source::Output);

    template.write_template("account_renew_account.json");
}

test_with_template!(test_account_renew, "account_renew_account.json");

challenge_with_generator!(
    challenge_account_renew_with_das_lock,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("renew_account", None);

        template.push_contract_cell("income-cell-type", false);
        template.push_contract_cell("balance-cell-type", false);

        template.push_oracle_cell(1, OracleCellType::Quote, 1000);
        template.push_config_cell(DataType::ConfigCellPrice, true, 0, Source::CellDep);

        let account = "das00001.bit";
        let next_account = "das00014.bit";
        let registered_at = timestamp - 86400;
        let expired_at = timestamp + 31536000 - 86400;

        // inputs
        let (cell_data, old_entity) =
            template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            20_000_000_000,
            Source::Input,
        );

        template.push_das_lock_cell(
            "0x030000000000000000000000000000000000004444",
            500_000_000_000,
            Source::Input,
            None,
        );

        let income_records = vec![IncomeRecordParam {
            belong_to: "0x0000000000000000000000000000000000000000".to_string(),
            capacity: 20_000_000_000,
        }];
        let (cell_data, entity) =
            template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(cell_data, Some((1, 1, entity)), 20_000_000_000, Source::Input);

        // outputs
        let (cell_data, new_entity) = template.gen_account_cell_data(
            account,
            next_account,
            registered_at,
            expired_at + 86400 * 365,
            0,
            0,
            0,
            None,
        );
        template.push_account_cell::<AccountCellData>(
            "0x0000000000000000000000000000000000001111",
            "0x0000000000000000000000000000000000002222",
            cell_data,
            None,
            20_000_000_000,
            Source::Output,
        );
        template.push_witness::<AccountCellData, AccountCellData, AccountCellData>(
            DataType::AccountCellData,
            Some((2, 0, new_entity)),
            Some((2, 0, old_entity)),
            None,
        );

        let income_records = vec![
            IncomeRecordParam {
                belong_to: "0x0000000000000000000000000000000000000000".to_string(),
                capacity: 20_000_000_000,
            },
            // Profit to DAS
            IncomeRecordParam {
                belong_to: "0x0300000000000000000000000000000000000000".to_string(),
                capacity: 500_000_000_000,
            },
        ];
        let (cell_data, entity) =
            template.gen_income_cell_data("0x0000000000000000000000000000000000000000", income_records);
        template.push_income_cell(cell_data, Some((1, 1, entity)), 520_000_000_000, Source::Output);

        template.as_json()
    }
);

#[test]
fn gen_account_recycle_expired_account_by_keeper() {
    let (mut template, timestamp) = init("recycle_expired_account_by_keeper", None);

    let account = "das00001.bit";
    let next_account = "das00014.bit";
    let registered_at = timestamp - 86400 * (365 + 30); // Register at 1 year and 1 month before
    let expired_at = timestamp - 86400 * 30 - 1; // Expired at 1 month + 1 second before

    let (cell_data, old_entity) =
        template.gen_account_cell_data(account, next_account, registered_at, expired_at, 0, 0, 0, None);
    template.push_account_cell::<AccountCellData>(
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

// test_with_template!(
//     test_account_recycle_expired_account_by_keeper,
//     "account_recycle_expired_account_by_keeper.json"
// );
