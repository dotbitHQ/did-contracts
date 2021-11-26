use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

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

#[test]
fn test_offer_make_offer() {
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

    test_tx(template.as_json());
}
