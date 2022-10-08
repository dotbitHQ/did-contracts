use das_types_std::constants::{DataType, Source};
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("collect_sub_account_profit", None);
    template.push_contract_cell("balance-cell-type", ContractType::Contract);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);

    push_dep_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
            }
        }),
    );

    template
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
            }
        }),
    );
}

#[test]
fn test_sub_account_collect_profit() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_collect_profit_modify_root() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    let current_root = [1u8; 32];
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                // Simulate modifying the root of the SubAccountCell.
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": 0,
                "owner_profit": 0,
            }
        }),
    );
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    challenge_tx(template.as_json(), ErrorCode::SubAccountCellConsistencyError)
}

#[test]
fn challenge_sub_account_collect_profit_parent_mismatch() {
    let mut template = before_each();

    // inputs
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_2
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": 1000_00_000_000u64,
                "owner_profit": 1000_00_000_000u64,
            }
        }),
    );

    // outputs
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_2
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": 1000_00_000_000u64,
                "owner_profit": 1000_00_000_000u64,
            }
        }),
    );
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellIdNotMatch)
}

#[test]
fn challenge_sub_account_not_collect_profit() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_sub_account_no_profit_to_collect() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0, 0);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_sub_account_collect_das_profit_incomplete() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    // Simulate not collecting all profit of DAS at once.
    push_simple_output_sub_account_cell(&mut template, 1, 0);
    push_output_normal_cell(&mut template, 1000_00_000_000 - 1, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    challenge_tx(template.as_json(), ErrorCode::SubAccountCollectProfitError)
}

#[test]
fn challenge_sub_account_collect_das_profit_error() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    // Simulate not transferring all profit to DAS.
    push_output_normal_cell(&mut template, 1000_00_000_000 - 1, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_sub_account_collect_das_profit_error_2() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    // Simulate not transferring all profit to other lock.
    push_output_normal_cell(
        &mut template,
        1000_00_000_000 - 1,
        "0x030000000000000000000000000000000000FFFF",
    );
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER);

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_sub_account_collect_owner_profit_incomplete() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    // Simulate not collecting all profit of owner at once.
    push_simple_output_sub_account_cell(&mut template, 0, 1);
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    push_output_balance_cell(&mut template, 1000_00_000_000 - 1, OWNER);

    challenge_tx(template.as_json(), ErrorCode::SubAccountCollectProfitError)
}

#[test]
fn challenge_sub_account_collect_owner_profit_error() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    // Simulate not transferring all profit to owner.
    push_output_balance_cell(&mut template, 1000_00_000_000 - 1, OWNER);

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_sub_account_collect_owner_profit_error_2() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    push_output_normal_cell(&mut template, 1000_00_000_000, DAS_WALLET_LOCK_ARGS);
    // Simulate not transferring all profit to other lock.
    push_output_balance_cell(&mut template, 1000_00_000_000, OWNER_1);

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}
