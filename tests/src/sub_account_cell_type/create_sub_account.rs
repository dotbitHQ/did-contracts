use super::common::*;
use crate::util::template_generator::SubAccountActionType;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
    push_input_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            }
        }),
    );
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                ]
            }
        }),
    );
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    template
}

#[test]
fn test_create_sub_account() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    let new_sub_account_cost = SUB_ACCOUNT_NEW_PRICE * template.sub_account_outer_witnesses.len() as u64;
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
    push_input_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": COMMON_INCOME_CREATOR
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": new_sub_account_cost
                    },
                ]
            }
        }),
    );
    push_input_normal_cell(&mut template, 10_000_000_000 - new_sub_account_cost, OWNER);

    test_tx(template.as_json())
}
