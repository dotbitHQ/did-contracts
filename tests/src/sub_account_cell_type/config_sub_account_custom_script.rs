use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("config_sub_account_custom_script", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(&mut template, "");

    template
}

fn push_simple_input_account_cell(template: &mut TemplateGenerator) {
    push_input_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, custom_script: &str) {
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": 0,
                "custom_script": custom_script
            }
        }),
    );
}

fn push_simple_output_account_cell(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, custom_script: &str) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": 0,
                "custom_script": custom_script
            }
        }),
    );
}

#[test]
fn test_sub_account_config_custom_script() {
    let mut template = before_each();

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(
        &mut template,
        "0x0116549cab7e92afb5f157141bc9da7781ce692a3144e47e2b8879a8d5a57b87c6",
    );

    test_tx(template.as_json())
}
