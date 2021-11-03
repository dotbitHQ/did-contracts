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

fn push_input_offer_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    account: &str,
    price: u64,
    message: &str,
) {
    template.push_input(
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

fn before_each() -> (TemplateGenerator, &'static str) {
    let mut template = init("cancel_offer");
    let owner = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_offer_cell(
        &mut template,
        200_100_000_000,
        owner,
        "xxxxx.bit",
        200_000_000_000,
        "Take my money.🍀",
    );

    (template, owner)
}

test_with_generator!(test_offer_cancel_offer, || {
    let (mut template, owner) = before_each();

    // inputs
    // Simulate canceling multiple OfferCells at once.
    push_input_offer_cell(
        &mut template,
        200_100_000_000,
        owner,
        "xxxxy.bit",
        200_000_000_000,
        "Take my money.🍀",
    );

    // outputs
    push_output_balance_cell(&mut template, 400199990000, owner);

    template.as_json()
});
