use das_types::constants::{AccountStatus, DataType, Source};
use serde_json::json;

use super::common::*;
use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::TemplateGenerator;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("create_approval", Some("0x00"));

    // inputs
    push_input_account_cell(&mut template, json!({}));

    template
}

#[test]
fn test_account_approval_create() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
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
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    test_tx(template.as_json())
}

#[test]
fn test_account_approval_create_edit_records() {
    let mut template = init("edit_records", Some("0x01"));
    template.push_config_cell(DataType::ConfigCellRecordKeyNamespace, Source::CellDep);

    // inputs
    push_input_account_cell_v4(
        &mut template,
        json!({
            "witness": {
                "last_edit_records_at": TIMESTAMP - DAY_SEC,
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
                        "protected_until": TIMESTAMP + DAY_SEC,
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
                "last_edit_records_at": TIMESTAMP,
                "records": [
                    // Simulate editing records after approved transferring.
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                ],
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
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

    // Manager actions should be able to pass even the AccountCell is approved to transfer.
    test_tx(template.as_json())
}

#[test]
fn challenge_account_approval_create_role_error() {
    // Simulate trying to push this transaction with manager role.
    let mut template = init("create_approval", Some("0x01"));

    // inputs
    push_input_account_cell(&mut template, json!({}));

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate not updating the status to ApprovedTransfer.
                "status": (AccountStatus::Normal as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellPermissionDenied)
}

#[test]
fn challenge_account_approval_create_spend_balance_cell() {
    let mut template = init("create_approval", Some("0x00"));

    // inputs
    push_input_account_cell(&mut template, json!({}));
    // Simulate spending BalanceCells in this transaction.
    push_input_balance_cell(&mut template, 100 * ONE_CKB, OWNER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate not updating the status to ApprovedTransfer.
                "status": (AccountStatus::Normal as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_account_approval_create_spend_fee_error() {
    let mut template = init("create_approval", Some("0x00"));

    // inputs
    push_input_account_cell(&mut template, json!({}));

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate spending fee more than the limit.
            "capacity": util::gen_account_cell_capacity(5) - 10001,
            "witness": {
                "status": (AccountStatus::Normal as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), ErrorCode::TxFeeSpentError)
}

#[test]
fn challenge_account_approval_create_status_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                // Simulate not updating the status to ApprovedTransfer.
                "status": (AccountStatus::Normal as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked)
}

#[test]
fn challenge_account_approval_create_account_near_expired() {
    let mut template = init("create_approval", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                // Simulate the account is near expired.
                "expired_at": TIMESTAMP + DAY_SEC * 30 - 1,
            },
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                // Simulate the account is near expired.
                "expired_at": TIMESTAMP + DAY_SEC * 30 - 1,
            },
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountHasNearGracePeriod)
}

#[test]
fn challenge_account_approval_create_approval_missing() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                // Simulate not providing the approval.
                "approval": null
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalActionUndefined)
}

#[test]
fn challenge_account_approval_create_action_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    // Simulate unsupported approval action.
                    "action": "xxxxxx",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalActionUndefined)
}

#[test]
fn challenge_account_approval_create_platform_lock_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    "action": "transfer",
                    "params": {
                        // Simulate the platform_lock is not a das-lock.
                        "platform_lock": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DUMMY_LOCK_ARGS
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::ApprovalParamsPlatformLockInvalid,
    )
}

#[test]
fn challenge_account_approval_create_protected_until_too_long() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
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
                        // Simulate the protected_until is too long.
                        "protected_until": TIMESTAMP + DAY_SEC * 10 + 1,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
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

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::ApprovalParamsProtectedUntilInvalid,
    )
}

#[test]
fn challenge_account_approval_create_sealed_until_too_long() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
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
                        "protected_until": TIMESTAMP + DAY_SEC,
                        // Simulate the sealed_until is too long.
                        "sealed_until": (TIMESTAMP + DAY_SEC) + DAY_SEC * 10 + 1,
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

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::ApprovalParamsSealedUntilInvalid,
    )
}

#[test]
fn challenge_account_approval_create_delay_count_invalid() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
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
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        // Simulate the delay_count_remain is not 1.
                        "delay_count_remain": 2,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::ApprovalParamsDelayCountRemainInvalid,
    )
}

#[test]
fn challenge_account_approval_create_to_lock_invalid() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
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
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        "delay_count_remain": 1,
                        // Simulate the to_lock is not a valid das-lock.
                        "to_lock": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DUMMY_LOCK_ARGS
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsToLockInvalid)
}
