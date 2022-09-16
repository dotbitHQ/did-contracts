use super::common::*;
use crate::util::{
    self, accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::{json, Value};

const OWNER_PROFIT: u64 = 20_000_000_000;
const DAS_PROFIT: u64 = 10_000_000_000;

pub fn push_input_account_cell_with_multi_sign(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": OWNER_1
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": ACCOUNT_1,
            "next": "yyyyy.bit",
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - 1,
        },
        "witness": {
            "account": ACCOUNT_1,
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8)
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_multi_sign_witness(0, 3, 5, "0x567419c40d0f2c3566e7630ee32697560fa97a7b543d8ec90d784f60cf920e76a359ae83839a5e7a14dd22136ce74aee2a007c71e5440143dab7b326619b019a75910e04d5f215ace571e5600d48b6766d6a5e1df00e2cf82dd4dcfbba444a94119ae2de");
}

pub fn push_input_account_cell_with_sub_account_and_multi_sign(template: &mut TemplateGenerator, cell_partial: Value) {
    let mut cell = json!({
        "capacity": util::gen_account_cell_capacity(5),
        "lock": {
            "owner_lock_args": OWNER_1,
            "manager_lock_args": OWNER_1
        },
        "type": {
            "code_hash": "{{account-cell-type}}"
        },
        "data": {
            "account": ACCOUNT_1,
            "next": "yyyyy.bit",
            "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - 1,
        },
        "witness": {
            "account": ACCOUNT_1,
            "registered_at": 0,
            "last_transfer_account_at": 0,
            "last_edit_manager_at": 0,
            "last_edit_records_at": 0,
            "status": (AccountStatus::Normal as u8),
            "enable_sub_account": 1,
        }
    });
    util::merge_json(&mut cell, cell_partial);

    template.push_input(cell, Some(2));
    template.push_multi_sign_witness(0, 3, 5, "0x567419c40d0f2c3566e7630ee32697560fa97a7b543d8ec90d784f60cf920e76a359ae83839a5e7a14dd22136ce74aee2a007c71e5440143dab7b326619b019a75910e04d5f215ace571e5600d48b6766d6a5e1df00e2cf82dd4dcfbba444a94119ae2de");
}

fn before_without_sub_account() -> TemplateGenerator {
    let mut template = init("confirm_expired_account_auction", None);

    // inputs
    push_input_account_cell_with_multi_sign(&mut template, json!({}));

    template
}

fn before_with_sub_account() -> TemplateGenerator {
    let mut template = init("confirm_expired_account_auction", None);

    template.push_contract_cell("sub-account-cell-type", false);

    // inputs
    push_input_account_cell_with_sub_account_and_multi_sign(&mut template, json!({}));
    push_input_sub_account_cell(
        &mut template,
        json!({
            "data": {
                "root": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "das_profit": DAS_PROFIT,
                "owner_profit": OWNER_PROFIT,
            }
        }),
    );

    template
}

#[test]
fn test_account_confirm_expired_account_auction() {
    let mut template = before_without_sub_account();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "data": {
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - 1,
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(5), OWNER_1);

    test_tx(template.as_json())
}

#[test]
fn test_account_confirm_expired_account_auction_with_sub_account() {
    let mut template = before_with_sub_account();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": OWNER_2
            },
            "data": {
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - 1,
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(5) + OWNER_PROFIT,
        OWNER_1,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    test_tx(template.as_json())
}
