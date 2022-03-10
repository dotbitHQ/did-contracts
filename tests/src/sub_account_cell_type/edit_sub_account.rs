use super::common::*;
use crate::util::{
    self, accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("edit_sub_account", Some("0x00"));

    template.restore_sub_account(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
    ]);

    // inputs
    push_input_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            }
        }),
    );

    template
}

#[test]
fn test_sub_account_edit() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "manager",
            // Simulate modifying manager.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "owner",
            // Simulate modifying owner.
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": u64::MAX,
            },
            "edit_key": "records",
            // Simulate modifying records.
            "edit_value": [
                {
                    "type": "address",
                    "key": "eth",
                    "label": "Personal",
                    "value": "0x0000000000000000000000000000000000000000",
                },
            ]
        }),
    );
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root)
            }
        }),
    );

    test_tx(template.as_json())
}
