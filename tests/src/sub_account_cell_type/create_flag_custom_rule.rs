use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

const USD_1: u64 = 1_000_000;
const USD_5: u64 = 5 * USD_1;
const USD_10: u64 = 10 * USD_1;
const USD_20: u64 = 20 * USD_1;

// total paid 100 USD
const TOTAL_PAID: u64 = USD_1 * 100 / CKB_QUOTE * ONE_CKB;

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_rules(&mut template);
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
                "status_flags": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_input_normal_cell(&mut template, TOTAL_PAID, OWNER);

    template
}

fn push_simple_outputs(template: &mut TemplateGenerator, total_profit: u64) {
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "das_profit": total_profit,
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flags": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(template, TOTAL_PAID - total_profit, OWNER);
}

fn push_simple_rules(template: &mut TemplateGenerator) {
    template.push_sub_account_rules_witness(
        DataType::SubAccountPreservedRule,
        1,
        json!(
            [
                {
                    "index": 0,
                    "name": "No emoji accounts",
                    "note": "",
                    "price": 0,
                    "status": 1,
                    "ast": {
                        "type": "function",
                        "name": "only_include_charset",
                        "arguments": [
                            {
                                "type": "variable",
                                "name": "account_chars",
                            },
                            {
                                "type": "value",
                                "value_type": "charset_type",
                                "value": "Emoji",
                            }
                        ],
                    }
                },
                {
                    "index": 1,
                    "name": "No preserved accounts",
                    "note": "",
                    "price": 0,
                    "status": 1,
                    "ast": {
                        "type": "function",
                        "name": "in_list",
                        "arguments": [
                            {
                                "type": "variable",
                                "name": "account",
                            },
                            {
                                "type": "value",
                                "value_type": "binary[]",
                                "value": [
                                    SUB_ACCOUNT_4_ID
                                ],
                            }
                        ],
                    }
                }
            ]
        ),
    );

    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(
            [
                {
                    "index": 0,
                    "name": "4 charactor account",
                    "note": "",
                    "price": USD_20, // 20 USD
                    "status": 0,
                    "ast": {
                        "type": "operator",
                        "symbol": "==",
                        "expressions": [
                            {
                                "type": "variable",
                                "name": "account_length",
                            },
                            {
                                "type": "value",
                                "value_type": "uint32",
                                "value": 4,
                            },
                        ],
                    }
                },
                {
                    "index": 1,
                    "name": "5 or more charactor account",
                    "note": "",
                    "price": USD_5, // 5 USD
                    "status": 1,
                    "ast": {
                        "type": "operator",
                        "symbol": ">=",
                        "expressions": [
                            {
                                "type": "variable",
                                "name": "account_length",
                            },
                            {
                                "type": "value",
                                "value_type": "uint32",
                                "value": 5,
                            },
                        ],
                    }
                },
                {
                    "index": 2,
                    "name": "special account",
                    "note": "",
                    "price": USD_10, // 10 USD
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
fn test_sub_account_create_flag_custom_rule_basic() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
            "edit_key": "manual",
            "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_1))
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 3);
    push_simple_outputs(&mut template, total_profit);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_create_flag_custom_rule_manual_mint() {
    let mut template = before_each();

    // outputs
    let smt = template.push_sub_account_mint_sign_witness(json!({
        "version": 1,
        "expired_at": TIMESTAMP + DAY_SEC,
        "account_list_smt_root": [
            [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
        ]
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_1)
    }));
    let total_profit = calculate_sub_account_cost(1);
    push_simple_outputs(&mut template, total_profit);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_create_flag_custom_rule_mix_mint() {
    let mut template = before_each();

    // outputs
    let smt = template.push_sub_account_mint_sign_witness(json!({
        "version": 1,
        "expired_at": TIMESTAMP + DAY_SEC,
        "account_list_smt_root": [
            [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
        ]
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "manual",
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_1)
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
    }));
    let mut total_profit = calculate_sub_account_cost(1);
    total_profit += util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_create_invalid_registered_at() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            // Simulate providing a registered_at that is not equal to timestamp in the TimeCell.
            "registered_at": TIMESTAMP - 1,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountInitialValueError,
    );
}

#[test]
fn challenge_sub_account_create_invalid_expired_at() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            // Simulate providing a expired_at that earlier than the registered_at.
            "expired_at": TIMESTAMP - 1,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountInitialValueError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_flag_not_consistent() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "das_profit": total_profit,
                "owner_profit": 0,
                // Simulate the flag is not consistent.
                "flag": SubAccountConfigFlag::CustomScript as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, TOTAL_PAID - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_owner_profit_not_consistent() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                // Simulate the owner profit is not consistent.
                "owner_profit": 1,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flags": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, TOTAL_PAID - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_price_rules_hash_not_consistent() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flags": SubAccountCustomRuleFlag::On as u8,
                // Simulate the price_rules_hash is not consistent.
                "price_rules_hash": BLACK_HOLE_HASH
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, TOTAL_PAID - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_preserve_rules_hash_not_consistent() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flags": SubAccountCustomRuleFlag::On as u8,
                // Simulate the preserve_rules_hash is not consistent.
                "preserved_rules_hash": BLACK_HOLE_HASH
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, TOTAL_PAID - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_mix_custom_script() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_script",
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid);
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_account_preserved() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            // Simulate try to register a preserved account.
            "account": SUB_ACCOUNT_4,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::AccountIsPreserved);
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_account_has_no_price() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            // Simulate try to register an account with no price.
            "account": "aaa.xxxxx.bit",
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::AccountHasNoPrice);
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_das_profit_incorrect() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": DUMMY_CHANNEL
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    // Simulate record incorrect das profit.
    push_simple_outputs(&mut template, total_profit - 1);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountProfitError);
}

#[test]
fn challenge_sub_account_create_flag_custom_rule_skipped() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            // Simulate try to register an account which only match the disabled rule.w
            "account": "0000.xxxxx.bit",
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_rule",
        "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
    }));

    let total_profit = util::usd_to_ckb(USD_5 * 1);
    push_simple_outputs(&mut template, total_profit);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::AccountHasNoPrice)
}

#[test]
fn perf_create_with_custom_rules() {
    let mut template = init_update();

    push_simple_dep_account_cell(&mut template);

    let chars = vec![serde_json::Value::String(String::from("0x0000000000000000000000000000000000000000")); 1001];
    let value = serde_json::Value::Array(chars);

    for i in 0..2 {
        template.push_sub_account_rules_witness(
            DataType::SubAccountPreservedRule,
            1,
            json!(
                [
                    {
                        "index": i,
                        "name": format!("Dummy rule {}", i),
                        "note": "A name that is not priced will not be automatically distributed;  name that meets more than one price rule may be automatically distributed at any one of the multiple prices it meets.",
                        "price": 0,
                        "status": 1,
                        "ast": {
                            "type": "operator",
                            "symbol": "and",
                            "expressions": [
                                {
                                    "type": "value",
                                    "value_type": "bool",
                                    "value": true,
                                },
                                {
                                    "type": "function",
                                    "name": "in_list",
                                    "arguments": [
                                        {
                                            "type": "variable",
                                            "name": "account",
                                        },
                                        {
                                            "type": "value",
                                            "value_type": "binary[]",
                                            "value": value
                                        },
                                    ]
                                }
                            ]
                        }
                    }
                ]
            ),
        );
    }

    let mut last_i = 0;
    let mut rules = vec![];
    for i in 0..50 {
        last_i = i;

        // generate a witness for every 20 rules
        if rules.len() > 20 {
            template.push_sub_account_rules_witness(
                DataType::SubAccountPriceRule,
                1,
                json!(rules),
            );

            rules = vec![];
        }

        rules.push(json!({
            "index": i,
            "name": format!("Dummy rule {}", i),
            "note": "A name that is not priced will not be automatically distributed;  name that meets more than one price rule may be automatically distributed at any one of the multiple prices it meets.",
            "price": 0,
            "status": 1,
            "ast": {
                "type": "operator",
                "symbol": "and",
                "expressions": [
                  {
                    "type": "operator",
                    "symbol": "==",
                    "expressions": [
                      {
                        "type": "variable",
                        "name": "account_length"
                      },
                      {
                        "type": "value",
                        "value_type": "uint32",
                        "value": 147
                      }
                    ]
                  },
                  {
                    "type": "function",
                    "name": "only_include_charset",
                    "arguments": [
                      {
                        "type": "variable",
                        "name": "account_chars"
                      },
                      {
                        "type": "value",
                        "value_type": "charset_type",
                        "value": "En"
                      }
                    ]
                  },
                  {
                    "type": "function",
                    "name": "include_words",
                    "arguments": [
                      {
                        "type": "variable",
                        "name": "account"
                      },
                      {
                        "type": "value",
                        "value_type": "string[]",
                        "value": [
                          "test1",
                          "test2",
                          "test3"
                        ]
                      }
                    ]
                  }
                ]
            }
        }))
    }
    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(rules),
    );

    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!([
            {
                "index": last_i + 1,
                "name": format!("Dummy rule last"),
                "note": "A name that is not priced will not be automatically distributed;  name that meets more than one price rule may be automatically distributed at any one of the multiple prices it meets.",
                "price": USD_5,
                "status": 1,
                "ast": {
                    "type": "operator",
                    "symbol": ">",
                    "expressions": [
                        {
                            "type": "variable",
                            "name": "account_length"
                        },
                        {
                            "type": "value",
                            "value_type": "uint8",
                            "value": 4
                        }
                    ]
                }
            }
        ]),
    );

    let total_paied = USD_5 * 100 / CKB_QUOTE * ONE_CKB;

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
                "status_flags": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_input_normal_cell(&mut template, TOTAL_PAID, OWNER);

    // outputs
    let account_count = 50;
    for i in 0..account_count {
        template.push_sub_account_witness_v2(json!({
            "action": SubAccountAction::Create.to_string(),
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": format!("test{}", i) + SUB_ACCOUNT_SUFFIX,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            },
            "edit_key": "custom_rule",
            "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
        }));
    }


    let total_profit = util::usd_to_ckb(USD_5 * account_count);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "das_profit": total_profit,
                "owner_profit": 0,
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flags": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, total_paied - total_profit, OWNER);

    test_tx(template.as_json())
}
