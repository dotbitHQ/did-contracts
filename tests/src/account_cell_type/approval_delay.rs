use das_types_std::constants::AccountStatus;
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
    let mut template = init("delay_approval", Some("0x00"));

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

    template
}

#[test]
fn test_account_approval_delay() {
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
                        "delay_count_remain": 0,
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
fn challenge_account_approval_delay_action_modified() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "witness": {
                "status": (AccountStatus::ApprovedTransfer as u8),
                "approval": {
                    // Simulate modifying the action of the approval.
                    "action": "xxxxxx",
                    "params": {
                        "platform_lock": {
                            "owner_lock_args": CHANNEL,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_account_approval_delay_platform_lock_modified() {
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
                        // Simulate modifying the platform_lock of the approval.
                        "platform_lock": {
                            "owner_lock_args": INVITER,
                            "manager_lock_args": CHANNEL
                        },
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_account_approval_delay_protected_until_modified() {
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
                        // Simulate modifying the action of the approval.
                        "protected_until": TIMESTAMP + DAY_SEC + 1,
                        "sealed_until": TIMESTAMP + DAY_SEC * 3,
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}

#[test]
fn challenge_account_approval_delay_sealed_until_not_increased() {
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
                        // Simulate not modifying the sealed_until of the approval.
                        "sealed_until": TIMESTAMP + DAY_SEC * 2,
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsSealedUntilIncrementError)
}

#[test]
fn challenge_account_approval_delay_sealed_until_increased_too_long() {
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
                        // Simulate increasing the sealed_until to too long..
                        "sealed_until": TIMESTAMP + DAY_SEC * (2 + 10) + 1,
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsSealedUntilIncrementError)
}

#[test]
fn challenge_account_approval_delay_count_remain_is_empty() {
    let mut template = init("delay_approval", Some("0x00"));

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
                        "protected_until": TIMESTAMP + DAY_SEC,
                        "sealed_until": TIMESTAMP + DAY_SEC * 2,
                        // Simulate try to delay even the delay_count_remain is 0.
                        "delay_count_remain": 0,
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
                        "delay_count_remain": 0,
                        "to_lock": {
                            "owner_lock_args": OWNER_2,
                            "manager_lock_args": OWNER_2
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsDelayCountNotEnough)
}

#[test]
fn challenge_account_approval_delay_count_remain_not_decreased() {
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
                        // Simulate not decreasing the delay_remain_count.
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

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsDelayCountDecrementError)
}

#[test]
fn challenge_account_approval_delay_to_lock_modified() {
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
                        "delay_count_remain": 0,
                        // Simulate not decreasing the delay_remain_count.
                        "to_lock": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DUMMY_LOCK_ARGS
                        }
                    }
                }
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::ApprovalParamsCanNotBeChanged)
}
