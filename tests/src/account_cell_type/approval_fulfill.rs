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
    let mut template = init("fulfill_approval", Some("0x00"));

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
fn test_account_approval_fulfill() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                "approval": null
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_account_approval_fulfill_status_not_normal() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "witness": {
                // Simulate not reseting the AccountCell.witness.status to normal.
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalFulfillError)
}

#[test]
fn challenge_account_approval_fulfill_approval_not_clear() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                // Simulate not clearing the approval of the AccountCell.
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

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalFulfillError)
}

#[test]
fn challenge_account_approval_fulfill_transfer_target_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate transfer the owner to the lock not equal to the to_lock.
                "owner_lock_args": OWNER_3,
                "manager_lock_args": OWNER_3
            },
            "witness": {
                "status": (AccountStatus::Normal as u8),
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalFulfillError)
}

#[test]
fn challenge_account_approval_fulfill_not_clear_records() {
    let mut template = init("fulfill_approval", Some("0x00"));

    // inputs
    push_input_account_cell_v4(
        &mut template,
        json!({
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
                        "key": "60",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ],
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

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "witness": {
                // Simulate not clearing the records of the AccountCell.
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                    {
                        "type": "address",
                        "key": "60",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ],
                "status": (AccountStatus::Normal as u8),
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalFulfillError)
}
