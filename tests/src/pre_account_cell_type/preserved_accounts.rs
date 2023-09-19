use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_parser::*;
use crate::util::{self};

#[test]
fn challenge_pre_register_preserved_account() {
    // Simulate registering an unavailable account.
    let account = "microsoft.bit";
    let mut template = init(json!({ "account": account }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 9, false),
            "witness": {
                "account": account,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountIsPreserved)
}

#[test]
fn test_pre_register_preserved_account_with_super_lock() {
    let account = "microsoft.bit";
    let mut template = init(json!({ "account": account, "has_super_lock": true }));

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a preserved account.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 9, false),
            "witness": {
                "account": account,
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
