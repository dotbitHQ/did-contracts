use das_types_std::constants::*;
use serde_json::{json, Value};

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
// use crate::util::error::*;
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
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    template
}

fn push_simple_sub_account_witness(template: &mut TemplateGenerator, sub_account_partial: Value) {
    let mut sub_account = json!({
        "action": SubAccountAction::CreateApproval.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
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

    template.push_sub_account_witness_v3(sub_account);
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
fn xxxx_sub_account_create_approval() {
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
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    test_tx(template.as_json())
}
