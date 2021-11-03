use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_input_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_balance_cell(template: &mut TemplateGenerator, capacity: u64, owner: &str) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}

fn push_output_offer_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    account: &str,
    price: u64,
    message: &str,
) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner,
            },
            "type": {
                "code_hash": "{{offer-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
                "message": message,
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000000001", None)
                },
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x050000000000000000000000000000000000000002", None)
                }
            }
        }),
        None,
    );
}

fn before_each(account: &str) -> (TemplateGenerator, u64, &'static str) {
    let mut template = init("make_offer");
    let owner = "0x050000000000000000000000000000000000001111";

    let account_without_suffix = &account[0..account.len() - 4];
    println!("account_without_suffix = {:?}", account_without_suffix);
    template.push_config_cell_derived_by_account(account_without_suffix, true, 0, Source::CellDep);

    // inputs
    let total_input = 300_000_000_000;
    push_input_balance_cell(&mut template, total_input / 3, owner);
    push_input_balance_cell(&mut template, total_input / 3, owner);
    push_input_balance_cell(&mut template, total_input / 3, owner);

    (template, total_input, owner)
}

test_with_generator!(test_offer_make_offer, || {
    let account = "xxxxx.bit";
    let (mut template, total_input, owner) = before_each(account);

    push_output_offer_cell(
        &mut template,
        200_100_000_000,
        owner,
        account,
        200_000_000_000,
        "Take my money.🍀",
    );
    push_output_balance_cell(&mut template, total_input - 200_000_000_000, owner);

    template.as_json()
});
