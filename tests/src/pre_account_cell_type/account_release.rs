use super::common::*;
use crate::util::{self, accounts::*, constants::*, error::Error, template_common_cell::*, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

#[test]
fn test_pre_register_shortest_registrable_account() {
    // Simulate registering the shortest registrable account for now.
    let account = "0j7p.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 4, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 4,
                    "new": ACCOUNT_PRICE_4_CHAR,
                    "renew": ACCOUNT_PRICE_4_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_3_chars_account_with_super_lock() {
    let account = "mc7.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a three chars account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 3, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 3,
                    "new": ACCOUNT_PRICE_3_CHAR,
                    "renew": ACCOUNT_PRICE_3_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_pre_register_3_chars_account() {
    // Simulate registering an unavailable account.
    let account = "mc7.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 3, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 3,
                    "new": ACCOUNT_PRICE_3_CHAR,
                    "renew": ACCOUNT_PRICE_3_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountStillCanNotBeRegister)
}

#[test]
fn test_pre_register_10_chars_account() {
    // The account with 10 or more charactors should always pass.
    let account = "1234567890.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a three chars account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_unreleased_account_with_super_lock() {
    // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
    let account = "g0xhlqew.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a unreleased account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_pre_register_unreleased_account() {
    // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
    let account = "g0xhlqew.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountStillCanNotBeRegister)
}
