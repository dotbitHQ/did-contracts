use super::common::*;
use crate::util::{
    self, accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // inputs
    push_input_simple_apply_register_cell(&mut template);

    template
}

fn push_input_simple_apply_register_cell(template: &mut TemplateGenerator) {
    push_input_apply_register_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_SP_1,
                "height": HEIGHT - 4,
                "timestamp": TIMESTAMP - 60,
            }
        }),
    );
}

fn push_output_simple_pre_account_cell(template: &mut TemplateGenerator) {
    push_output_pre_account_cell(
        template,
        json!({
            "capacity": util::gen_register_fee(8, true),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                "inviter_id": "0x0000000000000000000000000000000000000000",
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(INVITER, None)
                },
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(CHANNEL, None)
                },
                "invited_discount": INVITED_DISCOUNT
            }
        }),
    );
}

#[test]
fn test_pre_register_simple() {
    let mut template = before_each();

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_pre_register_apply_still_need_wait() {
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    push_input_apply_register_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_SP_1,
                // Simulate the ApplyRegisterCell is created just now.
                "height": HEIGHT,
                "timestamp": TIMESTAMP - 60,
            }
        }),
    );

    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), Error::ApplyRegisterNeedWaitLonger)
}

#[test]
fn challenge_pre_register_apply_timeout() {
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    push_input_apply_register_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_SP_1,
                // Simulate the ApplyRegisterCell is created far more ago.
                "height": HEIGHT - APPLY_MAX_WAITING_BLOCK - 1,
                "timestamp": TIMESTAMP - 60,
            }
        }),
    );

    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), Error::ApplyRegisterHasTimeout)
}

#[test]
fn challenge_pre_register_apply_hash_is_invalid() {
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    push_input_apply_register_cell(
        &mut template,
        json!({
            "data": {
                // Simulate the ApplyRegisterCell has different account with the PreAccountCell.
                "account": ACCOUNT,
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - 60,
            }
        }),
    );

    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), Error::PreRegisterApplyHashIsInvalid)
}

#[test]
fn challenge_pre_register_invalid_account_id() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "data": {
                // Simulate providing an invalid account ID with is not match the account in witness.
                "id": "0x0000000000000000000000000000000000000000"
            },
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterAccountIdIsInvalid)
}

#[test]
fn challenge_pre_register_created_at_mismatch() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                // Simulate the created_at field is not match with the TimeCell.
                "created_at": TIMESTAMP - 1,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterCreateAtIsInvalid)
}

#[test]
fn challenge_pre_register_invalid_owner_lock_args() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                // Simulate providing an invalid das-lock args.
                "owner_lock_args": "0x00"
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterOwnerLockArgsIsInvalid)
}

#[test]
fn challenge_pre_register_quote_mismatch() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                // Simulate the quote is not match with which in QuoteCell.
                "quote": CKB_QUOTE - 1,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterQuoteIsInvalid)
}

#[test]
fn challenge_pre_register_exceed_account_max_length() {
    // Simulate registering an account longer than maximum length limitation.
    let account = "1234567890123456789012345678901234567890123.bit";

    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // inputs
    push_input_apply_register_cell(
        &mut template,
        json!({
            "data": {
                "account": account,
                "height": HEIGHT - 4,
                "timestamp": TIMESTAMP - 60,
            }
        }),
    );

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(43, false),
            "witness": {
                "account": account,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterAccountIsTooLong)
}

#[test]
fn challenge_pre_register_discount_not_zero_when_no_inviter() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                // Simulate providing discount when no inviter or channel is listed.
                "invited_discount": INVITED_DISCOUNT,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterDiscountIsInvalid)
}

#[test]
fn challenge_pre_register_discount_incorrect() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                "inviter_id": "0x0000000000000000000000000000000000000000",
                "inviter_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(INVITER, None)
                },
                "channel_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(CHANNEL, None)
                },
                // Simulate providing incorrect discount.
                "invited_discount": INVITED_DISCOUNT - 1,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterDiscountIsInvalid)
}

#[test]
fn challenge_pre_register_incorrect_price() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee(8, false),
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                // Simulate providing price which is not match the account length.
                "price": {
                    "length": 4,
                    "new": ACCOUNT_PRICE_4_CHAR,
                    "renew": ACCOUNT_PRICE_4_CHAR
                },
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterPriceInvalid)
}

#[test]
fn challenge_pre_register_incorrect_capacity() {
    let mut template = before_each();

    push_output_pre_account_cell(
        &mut template,
        json!({
            // Simulate providing capacity less than one year.
            "capacity": util::gen_register_fee(8, false) - 1,
            "witness": {
                "account": ACCOUNT_SP_1,
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterCKBInsufficient)
}
