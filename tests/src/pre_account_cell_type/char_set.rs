use super::common::*;
use crate::util::{self, constants::*, error::Error, template_common_cell::*, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

#[test]
fn challenge_pre_register_invalid_char() {
    // Simulate registering an account with invalid character.
    // ⚠️ Need to delete the emoji from char_set_emoji.txt first, otherwise the test can not pass.
    let account = "✨das🎱001.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
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

    challenge_tx(template.as_json(), Error::PreRegisterAccountCharIsInvalid)
}

#[test]
fn challenge_pre_register_unsupported_char_set() {
    // Simulate registering an account with invalid character.
    // ⚠️ Need to delete the emoji from char_set_emoji.txt first, otherwise the test can not pass.
    let account = "✨das大001.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
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

    challenge_tx(template.as_json(), Error::PreRegisterFoundUndefinedCharSet)
}
