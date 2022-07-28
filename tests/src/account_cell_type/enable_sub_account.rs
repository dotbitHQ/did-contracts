use super::common::*;
use crate::util::{
    accounts::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*,
};
use serde_json::json;

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
fn test_enable_sub_account() {
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
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
        }),
    );
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    test_tx(template.as_json())
}

#[test]
fn challenge_enable_sub_account_beta_limit() {
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
                "account": "xxxx1.bit",
            },
            "witness": {
                "account": "xxxx1.bit",
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
                "account": "xxxx1.bit",
            },
            "witness": {
                "account": "xxxx1.bit",
                "enable_sub_account": 1,
            }
        }),
    );
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": "xxxx1.bit"
            },
        }),
    );
    push_output_balance_cell(&mut template, 479_000_000_000, SENDER);

    challenge_tx(template.as_json(), Error::SubAccountJoinBetaError);
}
