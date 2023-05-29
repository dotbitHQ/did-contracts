use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init_config("config_sub_account", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_input_sub_account_cell_v2(&mut template, json!({}), ACCOUNT_1);

    template
}

fn push_simple_input_account_cell(template: &mut TemplateGenerator) {
    push_input_account_cell(
        template,
        json!({
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_output_account_cell(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator) {
    push_output_sub_account_cell_v2(
        template,
        json!({
            "data": {
                "flag": SubAccountConfigFlag::CustomRule as u8,
                "status_flag": SubAccountCustomRuleFlag::On as u8,
            }
        }),
        ACCOUNT_1,
    );
}

#[test]
fn test_sub_account_config_manual() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "flag": SubAccountConfigFlag::Manual as u8,

            }
        }),
        ACCOUNT_1,
    );

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_config_custom_rule() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);

    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(
            [
                {
                    "index": 0,
                    "name": "Price of 1 Charactor Emoji DID",
                    "note": "",
                    "price": 100_000_000,
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
                                        "name": "account_length",
                                    },
                                    {
                                        "type": "value",
                                        "value_type": "uint32",
                                        "value": 1,
                                    },
                                ],
                            },
                            {
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
                        ]
                    }
                }
            ]
        ),
    );
    push_simple_output_sub_account_cell(&mut template);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_config_empty_custom_rule() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(&mut template);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_config_change_custom_script() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                "flag": SubAccountConfigFlag::CustomScript as u8,
                "custom_script": SCRIPT_CODE_HASH,
                "script_args": "",
            }
        }),
        ACCOUNT_1,
    );

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ConfigFlagInvalid)
}

#[test]
fn challenge_sub_account_config_custom_rule_invalid_syntax() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);

    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(
            [
                {
                    "index": 0,
                    "name": "Dummy rule",
                    "note": "",
                    "price": 100_000_000,
                    "status": 1,
                    "ast": {
                        "type": "value",
                        "value_type": "bool",
                        "value": false,
                    }
                }
            ]
        ),
    );
    push_simple_output_sub_account_cell(&mut template);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ConfigRulesHasSyntaxError);
}

// empty_base: 786041 cycle
#[test]
fn perf_empty_expression() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(&mut template);

    test_tx(template.as_json())
}

#[test]
fn perf_has_expression() {
    let mut template = before_each();

    let chars = vec![serde_json::Value::String(String::from("0x0000000000000000000000000000000000000000")); 1001];
    let value = serde_json::Value::Array(chars);

    for i in 0..2 {
        template.push_sub_account_rules_witness(
            DataType::SubAccountPriceRule,
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

    let mut rules = vec![];
    for i in 2..101 {
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

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(&mut template);

    let len = template.sub_account_outer_witnesses[0].len();
    println!("witness length: {}", (len - 2) / 2);

    test_tx(template.as_json())
}
