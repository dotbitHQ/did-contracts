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
            "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            // This account is in exipration grace period.
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            // This account is expired.
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    template
}

fn push_simple_sub_account_witness(template: &mut TemplateGenerator, sub_account_partial: Value) {
    let mut sub_account = json!({
        "action": SubAccountAction::Recycle.to_string(),
        "sub_account": {
            "suffix": SUB_ACCOUNT_SUFFIX,
        },
    });
    util::merge_json(&mut sub_account, sub_account_partial);

    template.push_sub_account_witness_v2(sub_account);
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_input_sub_account_cell_v2(
        template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
            }
        }),
        ACCOUNT_1,
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                "flag": SubAccountConfigFlag::CustomScript as u8,
            }
        }),
        ACCOUNT_1,
    );
}

#[test]
fn test_sub_account_recycle() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_recycle_when_parent_expired() {
    let mut template = init_update();

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "account": ACCOUNT_1,
                // Simulate the parent account is expired.
                "expired_at": TIMESTAMP - 1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );

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
            "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            // This account is in exipration grace period.
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            // This account is expired.
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    // outputs
    // This transaction only contains recycle sub action, so it should pass the verification even if the parent account
    // is expired.
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_recycle_account_not_expired() {
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
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                // Simulate recycling an account that is not expired.
                "expired_at": TIMESTAMP + YEAR_SEC,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::AccountStillCanNotBeRecycled,
    );
}

#[test]
fn challenge_sub_account_recycle_account_in_grace_period() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
                // Simulate recycling an account that is in expiration grace period.
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            },
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::AccountStillCanNotBeRecycled,
    );
}

#[test]
fn challenge_sub_account_recycle_smt_not_clear() {
    let mut template = before_each();

    // outputs
    push_simple_sub_account_witness(
        &mut template,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "registered_at": TIMESTAMP - YEAR_SEC - ACCOUNT_EXPIRATION_GRACE_PERIOD,
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
            },
            // Simulate not seting the leaf of the SMT to zero after recycling.
            // The edit_value here is used to pass the value of the SMT leaf.
            "edit_value": "0xFF00000000000000000000000000000000000000000000000000000000000000"
        }),
    );
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), ErrorCode::SMTProofVerifyFailed);
}
