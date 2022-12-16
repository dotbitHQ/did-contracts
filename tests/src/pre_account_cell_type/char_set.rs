use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_parser::*;
use crate::util::{self};

#[test]
fn challenge_pre_register_invalid_char() {
    // Simulate registering an account with invalid character.
    let account = "✨das🇫🇮001.bit";
    let mut template = before_each(account);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 8, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "d", "type": CharSetType::En as u32 },
                    { "char": "a", "type": CharSetType::En as u32 },
                    { "char": "s", "type": CharSetType::En as u32 },
                    { "char": "🇫🇮", "type": CharSetType::Emoji as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "1", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::AccountCharIsInvalid)
}

#[test]
fn challenge_pre_register_zh() {
    // Simulate registering an account with invalid character.
    let account = "✨das大001.bit";
    let mut template = before_each(account);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 8, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "d", "type": CharSetType::En as u32 },
                    { "char": "a", "type": CharSetType::En as u32 },
                    { "char": "s", "type": CharSetType::En as u32 },
                    { "char": "大", "type": CharSetType::ZhHans as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "1", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::ConfigIsPartialMissing)
}

#[test]
fn challenge_pre_register_multiple_language() {
    // Simulate registering an account with invalid character.
    let account = "✨лд지얕001.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetRu, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetKo, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 8, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "л", "type": CharSetType::Ru as u32 },
                    { "char": "д", "type": CharSetType::Ru as u32 },
                    { "char": "지", "type": CharSetType::Ko as u32 },
                    { "char": "얕", "type": CharSetType::Ko as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "1", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::CharSetIsConflict)
}

#[test]
fn test_pre_register_ja() {
    let account = "✨のロ00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetJa, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "の", "type": CharSetType::Ja as u32 },
                    { "char": "ロ", "type": CharSetType::Ja as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_ko() {
    let account = "✨지얕00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetKo, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "지", "type": CharSetType::Ko as u32 },
                    { "char": "얕", "type": CharSetType::Ko as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_ru() {
    let account = "✨лд00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetRu, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "л", "type": CharSetType::Ru as u32 },
                    { "char": "д", "type": CharSetType::Ru as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_th() {
    let account = "✨ฆี่จั00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetTh, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "ฆี่", "type": CharSetType::Th as u32 },
                    { "char": "จั", "type": CharSetType::Th as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_tr() {
    let account = "✨çö00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetTr, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "ç", "type": CharSetType::Tr as u32 },
                    { "char": "ö", "type": CharSetType::Tr as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_vi() {
    let account = "✨ăk00.bit";
    let mut template = before_each(account);
    template.push_config_cell(DataType::ConfigCellCharSetVi, Source::CellDep);

    push_input_simple_apply_register_cell(&mut template, account);

    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_register_fee_v2(account, 5, false),
            "witness": {
                "account": [
                    { "char": "✨", "type": CharSetType::Emoji as u32 },
                    { "char": "ă", "type": CharSetType::Vi as u32 },
                    { "char": "k", "type": CharSetType::Vi as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                    { "char": "0", "type": CharSetType::Digit as u32 },
                ],
                "created_at": 0,
                "price": {
                    "length": 5,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    test_tx(template.as_json())
}
