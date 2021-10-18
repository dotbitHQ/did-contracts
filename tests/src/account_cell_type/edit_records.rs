use super::common::init;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::{AccountStatus, DataType};
use serde_json::{json, Value};

fn push_input_account_cell(template: &mut TemplateGenerator, timestamp: u64, records: Value) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": timestamp + 31536000 - 86400,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": timestamp - 86400,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8),
                "records": records
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, timestamp: u64, records: Value) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x000000000000000000000000000000000000001111",
                "manager_lock_args": "0x000000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": timestamp + 31536000 - 86400,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": timestamp - 86400,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": timestamp,
                "status": (AccountStatus::Normal as u8),
                "records": records
            }
        }),
        Some(2),
    );
}

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("edit_records", Some("0x01"));

    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, true, 0, Source::CellDep);

    push_input_account_cell(
        &mut template,
        timestamp,
        json!([
            {
                "type": "address",
                "key": "eth",
                "label": "Personal",
                "value": "0x0000000000000000000000000000000000000000",
            },
            {
                "type": "address",
                "key": "eth",
                "label": "Company",
                "value": "0x0000000000000000000000000000000000001111",
            },
            {
                "type": "address",
                "key": "btc",
                "label": "Personal",
                "value": "0x0000000000000000000000000000000000002222",
            },
            {
                "type": "dweb",
                "key": "ipfs",
                "label": "Mars",
                "value": "0x00000000000000000000",
            },
            {
                "type": "profile",
                "key": "email",
                "label": "Company",
                "value": "0x00000000000000000000",
            },
            {
                "type": "custom_key",
                "key": "xxxx",
                "label": "xxxxxx",
                "value": "0x00000000000000000000",
            }
        ]),
    );

    (template, timestamp)
}

test_with_generator!(test_account_edit_records, || {
    let (mut template, timestamp) = before_each();

    push_output_account_cell(
        &mut template,
        timestamp,
        json!([
            {
                "type": "address",
                "key": "eth",
                "label": "Personal",
                "value": "0x0000000000000000000000000000000000000000",
            },
            {
                "type": "address",
                "key": "eth",
                "label": "Company",
                "value": "0x0000000000000000000000000000000000001111",
            }
        ]),
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_edit_records_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = before_each();

        // Simulate editing multiple AccountCells in one transaction.
        push_input_account_cell(&mut template, timestamp, json!([]));

        push_output_account_cell(&mut template, timestamp, json!([]));
        push_output_account_cell(&mut template, timestamp, json!([]));

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_edit_records_invalid_char,
    Error::AccountCellRecordKeyInvalid,
    || {
        let (mut template, timestamp) = before_each();

        // Simulate using invalid char in the key field of a record.
        push_output_account_cell(
            &mut template,
            timestamp,
            json!([
                {
                    "type": "address",
                    "key": "eth+",
                    "label": "Company",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]),
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_edit_records_invalid_key,
    Error::AccountCellRecordKeyInvalid,
    || {
        let (mut template, timestamp) = before_each();

        // Simulate using a key out of namespace.
        push_output_account_cell(
            &mut template,
            timestamp,
            json!([
                {
                    "type": "dweb",
                    "key": "xxxx",
                    "label": "xxxxx",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]),
        );

        template.as_json()
    }
);
