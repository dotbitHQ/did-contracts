use super::common::*;
use crate::util::{constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use das_types::constants::Source;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("edit_offer");

    let account_without_suffix = &ACCOUNT[0..ACCOUNT.len() - 4];
    println!("account_without_suffix = {:?}", account_without_suffix);
    template.push_config_cell_derived_by_account(account_without_suffix, true, 0, Source::CellDep);

    template
}

#[test]
fn test_offer_edit_offer_higher() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_input_balance_cell(&mut template, 200_000_000_000, BUYER);

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "300_099_990_000",
            "witness": {
                "account": ACCOUNT,
                "price": "300_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_output_balance_cell(&mut template, 100_000_000_000, BUYER);

    test_tx(template.as_json());
}

#[test]
fn test_offer_edit_offer_lower() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "100_099_990_000",
            "witness": {
                "account": ACCOUNT,
                "price": "100_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, 100_000_000_000, BUYER);

    test_tx(template.as_json());
}

#[test]
fn challenge_offer_edit_offer_create_cell() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "100_099_990_000",
            "witness": {
                "account": ACCOUNT,
                "price": "100_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate creating OfferCell when editing.
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "100_100_000_000",
            "witness": {
                "account": "yyyyy.bit",
                "price": "100_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_edit_offer_delete_cell() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    // Simulate deleting OfferCell when editing.
    push_output_balance_cell(&mut template, 200_099_990_000, BUYER);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_edit_offer_lower_capacity() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            // Simulate the capacity and the price is mismatched.
            "capacity": 100_099_990_000u64 - 1,
            "witness": {
                "account": ACCOUNT,
                "price": "100_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, 100_000_000_000 + 1, BUYER);

    challenge_tx(template.as_json(), Error::OfferCellCapacityError);
}

#[test]
fn challenge_offer_edit_offer_higher_capacity() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_input_balance_cell(&mut template, 200_000_000_000, BUYER);

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            // Simulate the capacity and the price is mismatched.
            "capacity": 300_099_990_000u64 - 1,
            "witness": {
                "account": ACCOUNT,
                "price": "300_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, 100_000_000_000 + 1, BUYER);

    challenge_tx(template.as_json(), Error::OfferCellCapacityError);
}

#[test]
fn challenge_offer_edit_offer_too_long_message() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                // Simulate the length of the message in bytes has reached the limit.
                "message": "Take my money.🍀".repeat(400)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::OfferCellMessageTooLong);
}

#[test]
fn challenge_offer_edit_offer_change_account() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                // Simulate the OfferCell.witness.account has been changed.
                "account": "yyyyy.bit",
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    challenge_tx(template.as_json(), Error::OfferCellFieldCanNotModified);
}

#[test]
fn challenge_offer_edit_offer_change_inviter_lock() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
                },
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                // Simulate the OfferCell.witness.inviter_lock has been changed.
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x057777000000000000000000000000000000007777", None)
                },
            }
        }),
    );

    challenge_tx(template.as_json(), Error::OfferCellFieldCanNotModified);
}

#[test]
fn challenge_offer_edit_offer_change_channel_lock() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
                },
            }
        }),
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                // Simulate the OfferCell.witness.channel_lock has been changed.
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args("0x058888000000000000000000000000000000008888", None)
                },
            }
        }),
    );

    challenge_tx(template.as_json(), Error::OfferCellFieldCanNotModified);
}

#[test]
fn challenge_offer_edit_offer_no_change() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    // Simulate the length of the message in bytes has reached the limit.
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_edit_offer_change_capacity() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_input_balance_cell(&mut template, 200_000_000_000, BUYER);

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "300_099_990_000",
            "witness": {
                "account": ACCOUNT,
                "price": "300_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_output_balance_cell(&mut template, 100_000_000_000 - 1, BUYER);

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_offer_edit_offer_change_owner() {
    let mut template = before_each();

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_input_balance_cell(&mut template, 200_000_000_000, BUYER);

    // outputs
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "300_099_990_000",
            "witness": {
                "account": ACCOUNT,
                "price": "300_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    push_output_balance_cell(
        &mut template,
        100_000_000_000,
        "0x058888000000000000000000000000000000008888",
    );

    challenge_tx(template.as_json(), Error::ChangeError);
}
