use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn before_each(account: &str) -> (TemplateGenerator, &'static str) {
    let mut template = init("edit_offer");
    let owner = "0x050000000000000000000000000000000000001111";

    let account_without_suffix = &account[0..account.len() - 4];
    println!("account_without_suffix = {:?}", account_without_suffix);
    template.push_config_cell_derived_by_account(account_without_suffix, true, 0, Source::CellDep);

    (template, owner)
}

test_with_generator!(test_offer_edit_offer_higher, || {
    let account = "xxxxx.bit";
    let (mut template, owner) = before_each(account);

    // inputs
    push_input_offer_cell(
        &mut template,
        200_100_000_000,
        owner,
        account,
        200_000_000_000,
        "Take my money.🍀",
    );
    push_input_balance_cell(&mut template, 200_000_000_000, owner);

    // outputs
    push_output_offer_cell(
        &mut template,
        300_100_000_000,
        owner,
        account,
        300_000_000_000,
        "Take my money.🍀",
    );
    push_output_balance_cell(&mut template, 100_000_000_000, owner);

    template.as_json()
});

test_with_generator!(test_offer_edit_offer_lower, || {
    let account = "xxxxx.bit";
    let (mut template, owner) = before_each(account);

    // inputs
    push_input_offer_cell(
        &mut template,
        200_100_000_000,
        owner,
        account,
        200_000_000_000,
        "Take my money.🍀",
    );

    // outputs
    push_output_offer_cell(
        &mut template,
        100_100_000_000,
        owner,
        account,
        100_000_000_000,
        "Take my money.🍀",
    );
    push_output_balance_cell(&mut template, 99_999_990_000, owner);

    template.as_json()
});
