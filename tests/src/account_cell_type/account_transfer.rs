use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::AccountStatus;
use serde_json::json;

test_with_generator!(test_account_transfer, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));
    let sender = "0x000000000000000000000000000000000000001111";
    let gainer = "0x000000000000000000000000000000000000002222";

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": sender,
                "manager_lock_args": sender
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": gainer,
                "manager_lock_args": gainer
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_transfer_account_multiple_cells,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));
        let sender = "0x000000000000000000000000000000000000001111";
        let gainer = "0x000000000000000000000000000000000000002222";

        // Simulate transferring multiple AccountCells at one time.
        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": sender,
                    "manager_lock_args": sender
                },
            }),
        );
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": sender,
                    "manager_lock_args": sender
                }
            }),
        );

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": gainer,
                    "manager_lock_args": gainer
                },
                "witness": {
                    "last_transfer_account_at": timestamp,
                }
            }),
        );
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": gainer,
                    "manager_lock_args": gainer
                },
                "witness": {
                    "last_transfer_account_at": timestamp,
                }
            }),
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_transfer_account_not_modified,
    Error::AccountCellOwnerLockShouldBeModified,
    || {
        let (mut template, timestamp) = init("transfer_account", Some("0x00"));
        let sender = "0x000000000000000000000000000000000000001111";

        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": sender,
                    "manager_lock_args": sender
                }
            }),
        );

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    // Simulate owner not change after the transaction
                    "owner_lock_args": sender,
                    "manager_lock_args": sender
                },
                "witness": {
                    "last_transfer_account_at": timestamp,
                }
            }),
        );

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_transfer_too_often, Error::AccountCellThrottle, || {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));
    let sender = "0x000000000000000000000000000000000000001111";
    let gainer = "0x000000000000000000000000000000000000002222";

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": sender,
                "manager_lock_args": sender
            },
            "witness": {
                // Simulate transferring multiple times in a day.
                "last_transfer_account_at": timestamp - 86400 + 1,
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate owner not change after the transaction
                "owner_lock_args": gainer,
                "manager_lock_args": gainer
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    template.as_json()
});
