use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
    ]);
    push_simple_rules(&mut template);

    template
}

fn push_simple_rules(template: &mut TemplateGenerator) {
    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(
            [
                // CAREFUL! This rule should not contain any of the sub-account appears in the following tests.
                {
                    "index": 0,
                    "name": "special account",
                    "note": "",
                    "price": 1_000_000,
                    "status": 1,
                    "ast": {
                        "type": "function",
                        "name": "include_chars",
                        "arguments": [
                            {
                                "type": "variable",
                                "name": "account",
                            },
                            {
                                "type": "value",
                                "value_type": "string[]",
                                "value": [
                                    "✨",
                                    "🌈",
                                ],
                            },
                        ],
                    }
                }
            ]
        ),
    );
}

#[test]
fn test_sub_account_renew_flag_manual_by_others() {
    let mut template = before_each();

    // inputs
    push_input_sub_account_cell_v2(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": 0,
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::Manual as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER_4);

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
        }
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC * 2,
        }
    }));
    push_common_output_cells(&mut template, 3);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_renew_flag_custom_rule_by_others() {
    let mut template = before_each();

    // inputs
    push_input_sub_account_cell_v2(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "das_profit": 0,
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flag": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER_4);

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
        }
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC * 2,
        }
    }));
    let das_profit = calculate_sub_account_cost(3);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "das_profit": das_profit,
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flag": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, 10_000_000_000 - das_profit, OWNER_4);

    test_tx(template.as_json())
}
