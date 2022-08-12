use super::common::*;
use crate::util::{self, constants::*, error::Error, template_common_cell::*, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

#[test]
fn challenge_pre_register_invalid_char() {
    // Simulate registering an account with invalid character.
    let account = "✨das🇫🇮001.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
fn challenge_pre_register_zh() {
    // Simulate registering an account with invalid character.
    let account = "✨das大001.bit";
    let mut template = init();
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::ConfigIsPartialMissing)
}

#[test]
fn challenge_pre_register_multiple_language() {
    // Simulate registering an account with invalid character.
    let account = "✨лд지얕001.bit";
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetRu, Source::CellDep);
    template.push_config_cell(DataType::ConfigCellCharSetKo, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
                "price": {
                    "length": 8,
                    "new": ACCOUNT_PRICE_5_CHAR,
                    "renew": ACCOUNT_PRICE_5_CHAR
                }
            }
        }),
    );

    challenge_tx(template.as_json(), Error::PreRegisterAccountCharSetConflict)
}

#[test]
fn test_pre_register_ja() {
    let account = "✨のロ00.bit";
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetJa, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetKo, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetRu, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetTh, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetTr, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
    let mut template = init();
    template.push_config_cell(DataType::ConfigCellCharSetVi, Source::CellDep);
    template.push_config_cell_derived_by_account(account, Source::CellDep);

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
                "created_at": TIMESTAMP,
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
