use das_types_std::constants::*;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    template
}

fn push_simple_sub_account_witness(template: &mut TemplateGenerator, sub_account_partial: Value) {
    let mut sub_account = json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    });
    util::merge_json(&mut sub_account, sub_account_partial);

    // Simulate upgrate the SubAccount version in this transaction.
    template.push_sub_account_witness_v2(sub_account);
}

#[test]
fn test_sub_account_approval_create() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_approval_create_edit_records() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v2(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "status": AccountStatus::ApprovedTransfer as u8,
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
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    // outputs
    template.push_sub_account_witness_v2(
        json!({
            "action": SubAccountAction::Edit.to_string(),
            "sign_role": "0x01",
            "sign_expired_at": TIMESTAMP,
            "old_sub_account_version": 2,
            "new_sub_account_version": 2,
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
                "status": AccountStatus::ApprovedTransfer as u8,
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
            },
            "edit_key": "records",
            "edit_value": [
                {
                    "type": "profile",
                    "key": "twitter",
                    "label": "xxxxx",
                    "value": "0x0000000000000000000000000000000000001111",
                }
            ]
        })
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_approval_create_role_error() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        // Simulate signing with manager which is not allowed.
        "sign_role": "0x01",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellPermissionDenied)
}

#[test]
fn challenge_sub_account_approval_create_spend_balance_cell() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    // Simulate spending BalanceCells in this transaction.
    push_input_balance_cell(&mut template, 100 * ONE_CKB, OWNER_1);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused)
}

#[test]
fn challenge_sub_account_approval_create_account_near_expired() {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            // Simulate the account is near expired.
            "expired_at": TIMESTAMP + DAY_SEC * 30 - 1,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "expired_at": TIMESTAMP + DAY_SEC * 30 - 1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::AccountHasNearGracePeriod)
}

#[test]
fn challenge_sub_account_approval_create_edit_key_error() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "xxxxx",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid)
}

#[test]
fn challenge_sub_account_approval_create_edit_value_is_null() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        // Simulate
        "edit_value": null
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), ErrorCode::WitnessStructureError)
}

#[test]
fn challenge_sub_account_approval_create_action_error() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalActionUndefined)
}

#[test]
fn challenge_sub_account_approval_create_platform_lock_error() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsPlatformLockInvalid)
}

#[test]
fn challenge_sub_account_approval_create_protected_until_too_long() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsProtectedUntilInvalid)
}

#[test]
fn challenge_sub_account_approval_create_sealed_until_too_long() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsSealedUntilInvalid)
}

#[test]
fn challenge_sub_account_approval_create_delay_count_invalid() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsDelayCountRemainInvalid)
}

#[test]
fn challenge_sub_account_approval_create_to_lock_invalid() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 1,
        "new_sub_account_version": 2,
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "approval",
        "edit_value": {
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
    }));
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ApprovalParamsToLockInvalid)
}
