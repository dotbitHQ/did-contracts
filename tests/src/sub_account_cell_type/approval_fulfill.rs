use das_types::constants::*;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn before_each() -> TemplateGenerator {
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
                    "protected_until": TIMESTAMP - DAY_SEC,
                    "sealed_until": TIMESTAMP,
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
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
        "action": SubAccountAction::FulfillApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
        "old_sub_account_version": 2,
        "new_sub_account_version": 2,
        "sub_account": {
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
                    "protected_until": TIMESTAMP - DAY_SEC,
                    "sealed_until": TIMESTAMP,
                    "delay_count_remain": 1,
                    "to_lock": {
                        "owner_lock_args": OWNER_2,
                        "manager_lock_args": OWNER_2
                    }
                }
            }
        },
    });
    util::merge_json(&mut sub_account, sub_account_partial);

    // Simulate upgrate the SubAccount version in this transaction.
    template.push_sub_account_witness_v2(sub_account);
}

#[test]
fn test_sub_account_approval_fulfill() {
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
fn challenge_sub_account_approval_fulfill_edit_key_not_empty() {
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
            // Simulate the edit_key is not empty.
            "edit_key": "approval",
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid)
}

#[test]
fn challenge_sub_account_approval_fulfill_edit_value_not_empty() {
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
            // Simulate the edit_value is not empty.
            "edit_value": {
                "action": "transfer",
                "params": {
                    // Simulate modifying the platform_lock of the approval.
                    "platform_lock": {
                        "owner_lock_args": INVITER,
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
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid)
}
