use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

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

#[test]
fn test_offer_cancel_offer() {
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

    test_tx(template.as_json());
}
