use serde_json::json;

use super::common::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_parser::*;
use crate::util::{self};

#[test]
fn challenge_pre_register_unavailable_accounts() {
    // Simulate registering an unavailable account.
    let account = "thiscantr.bit";
    let mut template = before_each(account);

    // inputs
    push_input_simple_apply_register_cell(&mut template, account);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 9, false),
            "witness": {
                "account": account,
                "created_at": 0,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountIsUnAvailable)
}

#[test]
fn test_pre_register_unavailable_accounts_below_all() {
    // Challenge if the index of ConfigCells will overflow
    let account = "🐭🐂🐯🐰🐲🐍🐎🐑🐒🐔🐶🐷.bit";
    let mut template = before_each(account);

    // inputs
    push_input_simple_apply_register_cell(&mut template, account);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 12, false),
            "witness": {
                "account": account,
                "created_at": 0,
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
