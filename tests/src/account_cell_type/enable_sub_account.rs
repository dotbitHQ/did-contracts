use serde_json::{json, Value};

use super::common::*;
use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::TemplateGenerator;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init_for_sub_account("enable_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            }
        }),
    );
    push_input_balance_cell(&mut template, 500_000_000_000, SENDER);

    template
}

#[test]
fn test_enable_sub_account_no_skip() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell(&mut template, Value::Null);
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    test_tx(template.as_json())
}

#[test]
fn test_enable_sub_account_skip_verification() {
    let mut template = init_for_sub_account("enable_sub_account", Some("0x00"));
    let account = "12345678.bit";

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
            }
        }),
    );
    push_input_balance_cell(&mut template, 500_000_000_000, SENDER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell_v2(&mut template, Value::Null, account);
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    test_tx(template.as_json())
}

#[test]
fn challenge_enable_sub_account_account_expired() {
    let mut template = init_for_sub_account("enable_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "data": {
                // Simulate the account has been in expiration grace period.
                "expired_at": TIMESTAMP - 1,
            },
        }),
    );
    push_input_balance_cell(&mut template, 500_000_000_000, SENDER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell(&mut template, Value::Null);
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellInExpirationGracePeriod,
    )
}

#[test]
fn challenge_enable_sub_account_account_capacity_decreased() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate the capacity of the AccountCell is decreased.
            "capacity": util::gen_account_cell_capacity(5) - 1,
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell(&mut template, Value::Null);
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellChangeCapacityError)
}

#[test]
fn challenge_enable_sub_account_account_modified() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "witness": {
                // Simulate the account is modified.
                "account": ACCOUNT_2,
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell(&mut template, Value::Null);
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellProtectFieldIsModified,
    )
}
