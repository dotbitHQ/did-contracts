use super::common::*;
use crate::util::{constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use serde_json::json;

pub const MAKE_OFFER_COST: u64 = PRICE + OFFER_PREPARED_FEE_CAPACITY + SECONDARY_MARKET_COMMON_FEE;

fn before_each() -> (TemplateGenerator, u64) {
    let mut template = init("make_offer");

    let account_without_suffix = &ACCOUNT[0..ACCOUNT.len() - 4];
    // println!("account_without_suffix = {:?}", account_without_suffix);
    template.push_config_cell_derived_by_account(account_without_suffix, true, 0, Source::CellDep);

    // inputs
    let total_input = 600_000_000_000;
    push_input_balance_cell(&mut template, total_input / 3, BUYER);
    push_input_balance_cell(&mut template, total_input / 3, BUYER);
    push_input_balance_cell(&mut template, total_input / 3, BUYER);

    (template, total_input)
}

#[test]
fn test_offer_make_offer() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": PRICE + OFFER_PREPARED_FEE_CAPACITY,
            "witness": {
                "account": ACCOUNT,
                "price": PRICE,
                "message": "Take my money.🍀"
            }
        }),
    );

    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST, BUYER);

    test_tx(template.as_json());
}

#[test]
fn challenge_offer_make_offer_change_capacity() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": 200_100_000_000u64,
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate transfer changes less than the user should get.
    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST - 1, BUYER);

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_offer_make_offer_change_owner() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": 200_100_000_000u64,
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate transfer changes less than the user should get.
    push_output_balance_cell(
        &mut template,
        total_input - MAKE_OFFER_COST,
        "0x050000000000000000000000000000000000003333",
    );

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_offer_make_offer_create_multiple() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": PRICE + OFFER_PREPARED_FEE_CAPACITY,
            "witness": {
                "account": ACCOUNT,
                "price": PRICE,
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate creating multiple OfferCells at once.
    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": PRICE + OFFER_PREPARED_FEE_CAPACITY,
            "witness": {
                "account": "yyyyy.bit",
                "price": PRICE,
                "message": "Take my money.🍀"
            }
        }),
    );

    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST * 2, BUYER);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_make_offer_lower_capacity() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            // Simulate the capacity and the price is mismatched.
            "capacity": 200_100_000_000u64 - 1,
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST + 1, BUYER);

    challenge_tx(template.as_json(), Error::OfferCellCapacityError);
}

#[test]
fn challenge_offer_make_offer_higher_capacity() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            // Simulate the capacity and the price is mismatched.
            "capacity": 200_100_000_000u64 + 1,
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST - 1, BUYER);

    challenge_tx(template.as_json(), Error::OfferCellCapacityError);
}

#[test]
fn challenge_offer_make_offer_too_long_message() {
    let (mut template, total_input) = before_each();

    push_output_offer_cell(
        &mut template,
        json!({
            "capacity": 200_100_000_000u64,
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                // Simulate the length of the message in bytes has reached the limit.
                "message": "Take my money.🍀".repeat(400)
            }
        }),
    );
    push_output_balance_cell(&mut template, total_input - MAKE_OFFER_COST, BUYER);

    challenge_tx(template.as_json(), Error::OfferCellMessageTooLong);
}
