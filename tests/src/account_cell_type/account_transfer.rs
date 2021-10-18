use super::common::init;
use crate::util::{self, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::AccountStatus;
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, timestamp: u64, owner: &str, manager: &str) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager
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
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, timestamp: u64, owner: &str, manager: &str) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager
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
                "last_transfer_account_at": timestamp,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

test_with_generator!(test_account_transfer, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    push_input_account_cell(
        &mut template,
        timestamp,
        "0x000000000000000000000000000000000000001111",
        "0x000000000000000000000000000000000000001111",
    );
    push_output_account_cell(
        &mut template,
        timestamp,
        "0x000000000000000000000000000000000000002222",
        "0x000000000000000000000000000000000000002222",
    );

    template.as_json()
});

test_with_generator!(test_account_transfer_with_eip712, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    push_input_account_cell(
        &mut template,
        timestamp,
        "0x051100000000000000000000000000000000001111",
        "0x052200000000000000000000000000000000002222",
    );
    push_output_account_cell(
        &mut template,
        timestamp,
        "0x050000000000000000000000000000000000002222",
        "0x050000000000000000000000000000000000002222",
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_transfer_account_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));

        // Simulate transferring multiple AccountCells at one time.
        push_input_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000001111",
            "0x000000000000000000000000000000000000001111",
        );
        push_input_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000001111",
            "0x000000000000000000000000000000000000001111",
        );

        push_output_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000002222",
        );
        push_output_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000002222",
            "0x000000000000000000000000000000000000002222",
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_transfer_account_not_modified,
    Error::AccountCellOwnerLockShouldBeModified,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));

        // Simulate owner not change after the transaction
        push_input_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000001111",
            "0x000000000000000000000000000000000000001111",
        );

        push_output_account_cell(
            &mut template,
            timestamp,
            "0x000000000000000000000000000000000000001111",
            "0x000000000000000000000000000000000000001111",
        );

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_transfer_too_often, Error::AccountCellThrottle, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

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
                // Simulate the throttle
                "last_transfer_account_at": timestamp - 86400 + 1,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        timestamp,
        "0x050000000000000000000000000000000000002222",
        "0x050000000000000000000000000000000000002222",
    );

    template.as_json()
});
