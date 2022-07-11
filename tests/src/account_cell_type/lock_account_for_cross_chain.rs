use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init(
        "lock_account_for_cross_chain",
        Some("0x0000000000000011000000000000002200"),
    );

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    template
}

#[test]
fn test_account_lock_account_for_cross_chain() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_account_lock_account_for_cross_chain_account_multiple_cells() {
    let mut template = init(
        "lock_account_for_cross_chain",
        Some("0x0000000000000011000000000000002200"),
    );

    // Simulate locking multiple AccountCells at one time.
    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
        }),
    );
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_account_with_other_cells() {
    let mut template = init(
        "lock_account_for_cross_chain",
        Some("0x0000000000000011000000000000002200"),
    );

    template.push_contract_cell("balance-cell-type", false);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );
    // Simulate transferring some balance of the user at the same time.
    push_input_balance_cell(&mut template, 100_000_000_000, SENDER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_account_modified() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate owner changed after the transaction
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_not_clear_records() {
    let mut template = init(
        "lock_account_for_cross_chain",
        Some("0x0000000000000011000000000000002200"),
    );

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "records": [
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
                ]
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8),
                // Simulate not clearing all records when transferring.
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellRecordNotEmpty)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_data_account() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "data": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_data_next() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "data": {
                // Simulate the next field has been modified accidentally.
                "next": "ooooo.bit",
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_data_expired_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "data": {
                // Simulate the expired_at field has been modified accidentally.
                "expired_at": TIMESTAMP + YEAR_SEC * 2,
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_account() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_registered_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                // Simulate the registered_at field has been modified accidentally.
                "registered_at": 1234,
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_last_transfer_account_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8),
                // Simulate the last_transfer_account_at field has been modified accidentally.
                "last_transfer_account_at": 1234
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_last_edit_manager_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8),
                // Simulate the last_edit_manager_at field has been modified accidentally.
                "last_edit_manager_at": 1234
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_last_edit_records_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8),
                // Simulate the last_edit_records_at field has been modified accidentally.
                "last_edit_records_at": 1234
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_is_near_expired() {
    let mut template = init(
        "lock_account_for_cross_chain",
        Some("0x0000000000000011000000000000002200"),
    );
    let expired_at = TIMESTAMP + 30 * DAY_SEC - 1;

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "data": {
                // Simulate the interval between current time and the expiration is less than 90 days.
                "expired_at": expired_at,
            },
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "data": {
                "expired_at": expired_at,
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::CrossChainLockError)
}

#[test]
fn challenge_account_lock_account_for_cross_chain_modify_witness_status_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                // Simulate the status field has been changed to wrong value.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::CrossChainLockError)
}
