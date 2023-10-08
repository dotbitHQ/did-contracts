use das_types::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::since_util::SinceFlag;
use crate::util::template_common_cell::*;
use crate::util::template_generator::gen_since;
use crate::util::template_parser::*;
use crate::util::{self};

#[test]
fn test_pre_register_shortest_registrable_account() {
    // Simulate registering the shortest registrable account for now.
    let account = "0j7p.bit";
    let mut template = init(json!({ "account": account, "has_super_lock": true }));

    push_input_simple_apply_register_cell(&mut template, account);
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 4, false),
            "witness": {
                "account": account,
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
    let mut template = init(json!({ "account": account, "has_super_lock": true }));

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a three chars account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 3, false),
            "witness": {
                "account": account,
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
    let mut template = init(json!({ "account": account }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 3, false),
            "witness": {
                "account": account,
                "price": {
                    "length": 3,
                    "new": ACCOUNT_PRICE_3_CHAR,
                    "renew": ACCOUNT_PRICE_3_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}

#[test]
fn test_pre_register_10_chars_account() {
    // The account with 10 or more charactors should always pass.
    let account = "1234567890.bit";
    let mut template = init(json!({ "account": account, "has_super_lock": true }));

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a three chars account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
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

#[test]
fn test_pre_register_unreleased_account_with_super_lock() {
    // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
    let account = "g0xhlqew.bit";
    let mut template = init(json!({ "account": account, "has_super_lock": true }));

    push_input_simple_apply_register_cell(&mut template, account);
    // Simulate manually minting a unreleased account with super lock.
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
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

#[test]
fn challenge_pre_register_unreleased_account() {
    // This account is not registrable, because its first 4 bytes in u32 is bigger than 3435973836.
    let account = "g0xhlqew.bit";
    let mut template = init(json!({ "account": account }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 10, false),
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

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}

#[test]
fn test_pre_register_pure_digit_account_after_20221018() {
    let account = "0004.bit";
    let mut template = init(json!({ "account": account, "timestamp": TIMESTAMP_20221018 }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 4, false),
            "witness": {
                "account": [
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "4", "type": CharSetType::Digit as u32 },
                ],
                "price": {
                    "length": 4,
                    "new": ACCOUNT_PRICE_4_CHAR,
                    "renew": ACCOUNT_PRICE_4_CHAR
                },
                "created_at": TIMESTAMP_20221018
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_pure_emoji_account_after_20221018() {
    let account = "🏹🏹🏹🏹.bit";
    let mut template = init(json!({ "account": account, "timestamp": TIMESTAMP_20221018 }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 4, false),
            "witness": {
                "account": [
                    { "char": "🏹", "type": CharSetType::Emoji as u32 },
                    { "char": "🏹", "type": CharSetType::Emoji as u32 },
                    { "char": "🏹", "type": CharSetType::Emoji as u32 },
                    { "char": "🏹", "type": CharSetType::Emoji as u32 },
                ],
                "price": {
                    "length": 4,
                    "new": ACCOUNT_PRICE_4_CHAR,
                    "renew": ACCOUNT_PRICE_4_CHAR
                },
                "created_at": TIMESTAMP_20221018
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_pre_register_pure_digit_account_before_20221018() {
    let account = "0004.bit";
    let mut template = init(json!({ "account": account }));

    push_input_apply_register_cell(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP_20221018 - 1,
            },
            "data": {
                "account": account
            }
        }),
        gen_since(SinceFlag::Relative, SinceFlag::Height, 1),
    );

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 4, false),
            "witness": {
                "account": [
                    // Simulate trying to register full released char-sets before 20221028.
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "4", "type": CharSetType::Digit as u32 },
                ],
                "price": {
                    "length": 4,
                    "new": ACCOUNT_PRICE_4_CHAR,
                    "renew": ACCOUNT_PRICE_4_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}

#[test]
fn challenge_pre_register_pure_digit_account_less_than_4_chars_after_20221018() {
    let account = "000.bit";
    let mut template = init(json!({ "account": account, "timestamp": TIMESTAMP_20221018 }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 3, false),
            "witness": {
                "account": [
                    // Simulate trying to register account less than 4 chars.
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "price": {
                    "length": 3,
                    "new": ACCOUNT_PRICE_3_CHAR,
                    "renew": ACCOUNT_PRICE_3_CHAR
                },
                "created_at": TIMESTAMP_20221018
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}

#[test]
fn challenge_pre_register_unreleased_pure_vi_account_after_20221018() {
    let account = "evwcu.bit";
    let mut template = init(json!({ "account": account, "timestamp": TIMESTAMP_20221018 }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    // Simulate trying to register account with not fully unreleased char-set.
                    { "char": "e", "type": CharSetType::Vi as u32 },
                    { "char": "v", "type": CharSetType::Vi as u32 },
                    { "char": "w", "type": CharSetType::Vi as u32 },
                    { "char": "c", "type": CharSetType::Vi as u32 },
                    { "char": "u", "type": CharSetType::Vi as u32 },
                ],
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                "created_at": TIMESTAMP_20221018
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}

#[test]
fn challenge_pre_register_unreleased_pure_en_account_after_20221018() {
    let account = "ftyht.bit";
    let mut template = init(json!({ "account": account, "timestamp": TIMESTAMP_20221018 }));

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    // Simulate trying to register account with not fully unreleased char-set.
                    { "char": "f", "type": CharSetType::En as u32 },
                    { "char": "t", "type": CharSetType::En as u32 },
                    { "char": "y", "type": CharSetType::En as u32 },
                    { "char": "h", "type": CharSetType::En as u32 },
                    { "char": "t", "type": CharSetType::En as u32 },
                ],
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                },
                "created_at": TIMESTAMP_20221018
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountStillCanNotBeRegister)
}
