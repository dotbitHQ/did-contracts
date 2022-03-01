use super::common::*;
use crate::util::{accounts::*, template_common_cell::*, template_generator::TemplateGenerator, template_parser::*};
use serde_json::json;

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init_for_sub_account("enable_sub_account", Some("0x00"));

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

    (template, timestamp)
}

#[test]
fn test_enable_sub_account() {
    let (mut template, timestamp) = before_each();

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
                "args": ACCOUNT
            },
        }),
    );
    push_output_balance_cell(&mut template, 100_000_000_000, SENDER);

    test_tx(template.as_json())
}
