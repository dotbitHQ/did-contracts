use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_input_reverse_record_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    manager: &str,
    account: &str,
) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager,
            },
            "type": {
                "code_hash": "{{reverse-record-cell-type}}"
            },
            "data": {
                "account": account
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str, manager: &str) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}

fn before_each() -> (TemplateGenerator, &'static str, &'static str) {
    let mut template = init("retract_reverse_record");
    let owner = "0x050000000000000000000000000000000000001111";
    let manager = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, manager, "xxxxx.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, manager, "yyyyy.bit");
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, manager, "zzzzz.bit");

    (template, owner, manager)
}

test_with_generator!(test_reverse_record_retract, || {
    let (mut template, owner, manager) = before_each();

    push_output_balance_cell(&mut template, 20_100_000_000 * 3 - 10_000, owner, manager);

    template.as_json()
});
