use super::common::*;
use crate::util::{self, accounts::*, constants::*, error::Error, template_common_cell::*, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

#[test]
fn challenge_pre_register_preserved_account() {
    // Simulate registering an unavailable account.
    let account = "microsoft.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(9, false),
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

    challenge_tx(template.as_json(), Error::AccountIsPreserved)
}

#[test]
fn test_pre_register_preserved_account_with_super_lock() {
    let account = "microsoft.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a preserved account.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(9, false),
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
