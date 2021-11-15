use super::common::*;
use crate::util::{
    constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*,
};
use ckb_testtool::context::Context;
use das_types::constants::AccountStatus;
use serde_json::json;

fn push_simple_output_income_cell(template: &mut TemplateGenerator) {
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "500_000_000_000"
                    }
                ]
            }
        }),
    );
}

fn before_each() -> (TemplateGenerator, u64, &'static str, &'static str) {
    let (mut template, timestamp) = init_for_renew("renew_account", None);
    let owner = "0x000000000000000000000000000000000000001111";
    let sender = "0x000000000000000000000000000000000000002222";

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner
            },
            "data": {
                "expired_at": timestamp
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, sender);

    (template, timestamp, owner, sender)
}

#[test]
fn test_account_renew() {
    let (mut template, timestamp, owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, sender);

    test_tx(template.as_json());
}

#[test]
fn challenge_account_renew_modify_owner() {
    let (mut template, timestamp, _owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the owner of the AccountCell was changed.
                "owner_lock_args": "0x000000000000000000000000000000000000003333",
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, sender);

    challenge_tx(template.as_json(), Error::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_renew_less_than_one_year() {
    let (mut template, timestamp, owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                // Simulate the increment of the expired_at is less than one year.
                "expired_at": timestamp + 31_536_000 - 1,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, sender);

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationMustLongerThanYear)
}

#[test]
fn challenge_account_renew_payment_less_than_one_year() {
    let (mut template, timestamp, owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        // Simulate a payment shortfall.
                        "capacity": (500_000_000_000u64 - 1).to_string()
                    }
                ]
            }
        }),
    );
    push_output_balance_cell(&mut template, 500_000_000_000, sender);

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationMustLongerThanYear)
}

#[test]
fn challenge_account_renew_payment_less_than_increment() {
    let (mut template, timestamp, owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                "expired_at": timestamp + 31_536_000 * 3,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        // Simulate a payment shortfall.
                        "capacity": 500_000_000_000u64.to_string()
                    }
                ]
            }
        }),
    );
    push_output_balance_cell(&mut template, 500_000_000_000, sender);

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationBiggerThanPayed)
}

#[test]
fn challenge_account_renew_change_amount() {
    let (mut template, timestamp, owner, sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000 - 1, sender);

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_renew_change_owner() {
    let (mut template, timestamp, owner, _sender) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": owner,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        500_000_000_000,
        "0x000000000000000000000000000000000000003333",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}
