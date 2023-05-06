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
    let mut template = init("collect_sub_account_channel_profit", None);
    template.push_config_cell(DataType::ConfigCellSubAccount, Source::CellDep);

    template
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
            }
        }),
    );
}

#[test]
fn test_sub_account_collect_channel_profit() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 1000_00_000_000);
    push_output_normal_cell(&mut template, 1000_00_000_000, DUMMY_LOCK_ARGS);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_collect_channel_collect_without_manager_lock() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    // Simulate collecting profit without the profit manager's signature.
    // push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 1000_00_000_000);
    push_output_normal_cell(&mut template, 1000_00_000_000, DUMMY_LOCK_ARGS);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ProfitManagerLockIsRequired)
}

#[test]
fn challenge_sub_account_collect_channel_profit_modify_root() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    let current_root = [1u8; 32];
    push_output_sub_account_cell_v2(
        &mut template,
        json!({
            "data": {
                // Simulate modifying the root of the SubAccountCell.
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": 0,
                "owner_profit": "1000_00_000_000",
            }
        }),
        ACCOUNT_1,
    );
    push_output_normal_cell(&mut template, 1000_00_000_000, DUMMY_LOCK_ARGS);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    )
}

#[test]
fn challenge_sub_account_collect_channel_modify_owner_profit() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    // Simulate modifying the SubAccountCell.data.owner_profit.
    push_simple_output_sub_account_cell(&mut template, 0, 1000_00_000_000 - 1);
    push_output_normal_cell(&mut template, 1000_00_000_000, DUMMY_LOCK_ARGS);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    )
}

#[test]
fn challenge_sub_account_collect_channel_no_profit_to_collect() {
    let mut template = before_each();

    // inputs
    // Simulate no profit to collect.
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ProfitIsEmpty)
}

#[test]
fn challenge_sub_account_collect_channel_not_collect_profit() {
    let mut template = before_each();

    // inputs
    push_simple_input_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    push_input_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    // outputs
    push_simple_output_sub_account_cell(&mut template, 1000_00_000_000, 1000_00_000_000);
    push_output_normal_cell(&mut template, ONE_CKB, PROFIT_LOCK_ARGS);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ProfitMustBeCollected)
}
