use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;
use crate::util::{self};

fn push_output_simple_pre_account_cell(template: &mut TemplateGenerator) {
    push_output_pre_account_cell(
        template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, true),
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
                "invited_discount": INVITED_DISCOUNT,
            }
        }),
    );
}

#[test]
fn test_pre_register_simple_v1() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell_v1(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, true),
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

    test_tx(template.as_json());
}

#[test]
fn test_pre_register_simple_v2() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell_v2(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, true),
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

    test_tx(template.as_json());
}

#[test]
fn test_pre_register_simple_v3() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_pre_register_initial_record_key_invalid() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, true),
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
                "invited_discount": INVITED_DISCOUNT,
                "initial_records": [
                    {
                        "type": "address",
                        // Simulate creating a Pre
                        "key": "xxxx",
                        "label": "Personal",
                        "value": OWNER_WITHOUT_TYPE,
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellRecordKeyInvalid)
}

#[test]
fn challenge_pre_register_apply_still_need_wait() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
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
        None,
    );

    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterNeedWaitLonger)
}

#[test]
fn challenge_pre_register_apply_timeout() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
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
        None,
    );

    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterHasTimeout)
}

#[test]
fn challenge_pre_register_apply_hash_is_invalid() {
    let mut template = before_each(ACCOUNT_1);

    // inputs
    // Simulate the ApplyRegisterCell has different account with the PreAccountCell.
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::ApplyHashMismatch)
}

#[test]
fn challenge_pre_register_invalid_account_id() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::AccountIdIsInvalid)
}

#[test]
fn challenge_pre_register_created_at_mismatch() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::CreateAtIsInvalid)
}

#[test]
fn challenge_pre_register_invalid_owner_lock_args() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::OwnerLockArgsIsInvalid)
}

#[test]
fn challenge_pre_register_quote_mismatch() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::QuoteIsInvalid)
}

#[test]
fn challenge_pre_register_exceed_account_max_length() {
    // Simulate registering an account longer than maximum length limitation.
    let account = "1234567890123456789012345678901234567890123.bit";
    let mut template = before_each(account);

    // inputs
    push_input_simple_apply_register_cell(&mut template, account);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 43, false),
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

    challenge_tx(template.as_json(), ErrorCode::AccountIsTooLong)
}

#[test]
fn challenge_pre_register_discount_not_zero_when_no_inviter() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(
        template.as_json(),
        PreAccountCellErrorCode::InviteeDiscountShouldBeEmpty,
    )
}

#[test]
fn challenge_pre_register_discount_incorrect() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::InviteeDiscountIsInvalid)
}

#[test]
fn challenge_pre_register_incorrect_price() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false),
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::PriceIsInvalid)
}

#[test]
fn challenge_pre_register_incorrect_capacity() {
    let mut template = before_each(ACCOUNT_SP_1);

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_pre_account_cell(
        &mut template,
        json!({
            // Simulate providing capacity less than one year.
            "capacity": util::gen_register_fee_v2(ACCOUNT_SP_1, 8, false) - 1,
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

    challenge_tx(template.as_json(), PreAccountCellErrorCode::CKBIsInsufficient)
}

#[test]
fn challenge_pre_register_refered_exact_account() {
    // Account ID of ACCOUNT_SP_1: 0xacfa8b68f77544e40abbb9daaaacc96621a3ee36
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "id": "0xacfa8b68f77544e40abbb9daaaacc96621a3ee36",
                // Simulate the refered AccountCell is the exact AccountCell of ACCOUNT_SP_1.
                "next": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            }
        }),
    );

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(
        template.as_json(),
        PreAccountCellErrorCode::AccountAlreadyExistOrProofInvalid,
    )
}

#[test]
fn challenge_pre_register_refered_next_account() {
    // Account ID of ACCOUNT_SP_1: 0xacfa8b68f77544e40abbb9daaaacc96621a3ee36
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "id": "0x0000000000000000000000000000000000000000",
                // Simulate the refered AccountCell is the previouse AccountCell of ACCOUNT_SP_1.
                "next": "0xacfa8b68f77544e40abbb9daaaacc96621a3ee36",
            }
        }),
    );

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(
        template.as_json(),
        PreAccountCellErrorCode::AccountAlreadyExistOrProofInvalid,
    )
}

#[test]
fn challenge_pre_register_refered_before_account() {
    // Account ID of ACCOUNT_SP_1: 0xacfa8b68f77544e40abbb9daaaacc96621a3ee36
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "id": "0x0000000000000000000000000000000000000000",
                "next": "0xacfa8b68f77544e40abbb9daaaacc96621a3ee35",
            }
        }),
    );

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(
        template.as_json(),
        PreAccountCellErrorCode::AccountAlreadyExistOrProofInvalid,
    )
}

#[test]
fn challenge_pre_register_refered_after_account() {
    // Account ID of ACCOUNT_SP_1: 0xacfa8b68f77544e40abbb9daaaacc96621a3ee36
    let mut template = init();
    template.push_config_cell_derived_by_account(ACCOUNT_SP_1, Source::CellDep);

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "id": "0xacfa8b68f77544e40abbb9daaaacc96621a3ee37",
                "next": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            }
        }),
    );

    // inputs
    push_input_simple_apply_register_cell(&mut template, ACCOUNT_SP_1);

    // outputs
    push_output_simple_pre_account_cell(&mut template);

    challenge_tx(
        template.as_json(),
        PreAccountCellErrorCode::AccountAlreadyExistOrProofInvalid,
    )
}
