use das_types_std::constants::AccountStatus;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::TemplateGenerator;
use crate::util::template_parser::*;
use crate::util::{self};

pub fn push_input_account_cell_with_multi_sign(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": SENDER,
            "manager_lock_args": SENDER
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": ACCOUNT_1,
            "next": "yyyyy.bit",
            "expired_at": u64::MAX,
        },
        "witness": {
            "account": ACCOUNT_1,
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::LockedForCrossChain as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, None, Some(2));
    template.push_multi_sign_witness(0, 3, 5, "0x567419c40d0f2c3566e7630ee32697560fa97a7b543d8ec90d784f60cf920e76a359ae83839a5e7a14dd22136ce74aee2a007c71e5440143dab7b326619b019a75910e04d5f215ace571e5600d48b6766d6a5e1df00e2cf82dd4dcfbba444a94119ae2de");
}

fn before_each() -> TemplateGenerator {
    let mut template = init("unlock_account_for_cross_chain", Some("0x00"));

    // inputs
    push_input_account_cell_with_multi_sign(&mut template, json!({}));

    template
}

#[test]
fn test_account_unlock_account_for_cross_chain_keep_owner() {
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
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_account_unlock_account_for_cross_chain_change_owner() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_account_multiple_cells() {
    let mut template = init("unlock_account_for_cross_chain", Some("0x00"));

    // Simulate unlocking multiple AccountCells at one time.
    // inputs
    push_input_account_cell(
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
    push_input_account_cell(
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

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_data_account() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_data_next() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the next field has been modified accidentally.
                "next": "ooooo.bit",
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_data_expired_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the expired_at field has been modified accidentally.
                "expired_at": TIMESTAMP + YEAR_SEC * 2,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_account() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_registered_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                // Simulate the registered_at field has been modified accidentally.
                "registered_at": 1234,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_last_transfer_account_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate the last_transfer_account_at field has been modified accidentally.
                "last_transfer_account_at": 1234
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_last_edit_manager_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate the last_edit_manager_at field has been modified accidentally.
                "last_edit_manager_at": 1234
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_last_edit_records_at() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate the last_edit_records_at field has been modified accidentally.
                "last_edit_records_at": 1234
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}

#[test]
fn challenge_account_unlock_account_for_cross_chain_modify_witness_status_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                // Simulate the status field has been changed to wrong value.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::CrossChainUnlockError)
}
