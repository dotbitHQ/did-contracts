use super::common::*;
use crate::util::{accounts::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("cancel_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT_1,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    template
}

#[test]
fn test_offer_cancel_single_offer() {
    let mut template = before_each();

    // outputs
    push_output_balance_cell(&mut template, 200_099_990_000, BUYER);

    test_tx(template.as_json());
}

#[test]
fn test_offer_cancel_multiple_offer() {
    let mut template = before_each();

    // inputs
    // Simulate canceling multiple OfferCells at once.
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": "xxxxy.bit",
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );

    // outputs
    push_output_balance_cell(&mut template, 400_199_990_000, BUYER);

    test_tx(template.as_json());
}

#[test]
fn challenge_offer_cancel_new_in_outputs() {
    let mut template = before_each();

    // outputs
    // Simulate creating new OfferCell
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": "xxxxy.bit",
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, 400199990000, BUYER);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_cancel_offer_change_capacity() {
    let mut template = before_each();

    // outputs
    push_output_balance_cell(&mut template, 200_099_990_000 - 1, BUYER);

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_offer_cancel_offer_change_owner() {
    let mut template = before_each();

    // outputs
    push_output_balance_cell(
        &mut template,
        200_099_990_000,
        "0x058888000000000000000000000000000000008888",
    );

    challenge_tx(template.as_json(), Error::ChangeError);
}
