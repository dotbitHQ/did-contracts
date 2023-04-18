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
    push_simple_input_sub_account_cell(&mut template);

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

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator) {
    push_input_sub_account_cell_v2(template, json!({}), ACCOUNT_1);
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

// #[test]
// fn challenge_sub_account_config_custom_script_not_change() {
//     let mut template = init_config("config_sub_account", Some("0x00"));

//     challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountCustomScriptError)
// }
