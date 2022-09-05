use super::common::*;
use crate::util::{accounts::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("config_sub_account_custom_script", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(&mut template, "", "");

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

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, custom_script: &str, script_args: &str) {
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
                "custom_script": custom_script,
                "script_args": script_args
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

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, custom_script: &str, script_args: &str) {
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
                "custom_script": custom_script,
                "script_args": script_args
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
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "",
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_config_custom_script_not_change() {
    let mut template = init_create("config_sub_account_custom_script", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "",
    );

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "",
    );

    challenge_tx(template.as_json(), Error::SubAccountCustomScriptError)
}

#[test]
fn test_sub_account_config_custom_script_args_change() {
    let mut template = init_create("config_sub_account_custom_script", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "0x0011223300",
    );

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "0x0044556600",
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_config_custom_script_args_not_change() {
    let mut template = init_create("config_sub_account_custom_script", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "0x0011223300",
    );

    // outputs
    push_simple_output_account_cell(&mut template);
    push_simple_output_sub_account_cell(
        &mut template,
        "0x010000000000000000000000000000746573742d637573746f6d2d736372697074",
        "0x0011223300",
    );

    challenge_tx(template.as_json(), Error::SubAccountCustomScriptError)
}
