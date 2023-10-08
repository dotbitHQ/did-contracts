use das_types::constants::AccountStatus;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::AccountCellErrorCode;
// use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::TemplateGenerator;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("revoke_approval", Some("0x00"));

    // inputs
    push_input_account_cell_v4(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP - 1,
                        "sealed_until": TIMESTAMP + DAY_SEC * 2,
                        "delay_count_remain": 1,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    template
}

#[test]
fn test_account_approval_revoke() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::Normal as u8),
                "approval": null
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_account_approval_revoke_status_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate not reseting the status of the AccountCell to normal.
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalNotRevoked)
}

#[test]
fn challenge_account_approval_revoke_approval_not_clear() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate not clearing the approval field.
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP - 1,
                        "sealed_until": TIMESTAMP + DAY_SEC * 2,
                        "delay_count_remain": 1,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalNotRevoked)
}

#[test]
fn challenge_account_approval_revoke_protected_approval() {
    let mut template = init("revoke_approval", Some("0x00"));

    // inputs
    push_input_account_cell_v4(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        // Simulate the approval is still in the protect period.
                        "protected_until": TIMESTAMP,
                        "sealed_until": TIMESTAMP + DAY_SEC * 2,
                        "delay_count_remain": 1,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::Normal as u8),
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalInProtectionPeriod)
}
