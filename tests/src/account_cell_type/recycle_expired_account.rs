use super::common::init;
use crate::util::{
    self, accounts::*, constants::*, error::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

pub const DAS_PROFIT: u64 = 10_000_000_000;
pub const OWNER_PROFIT: u64 = 10_000_000_000;

fn push_prev_account_cell(template: &mut TemplateGenerator) {
    push_input_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00002.bit",
            },
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_expired_account_cell(template: &mut TemplateGenerator) {
    push_input_account_cell(
        template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - ACCOUNT_EXPIRATION_AUCTION_CONFIRMATION_PERIOD - 1,
            },
            "witness": {
                "account": "das00002.bit",
                "enable_sub_account": 1,
            }
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_expired_sub_account_cell(template: &mut TemplateGenerator) {
    push_input_sub_account_cell(
        template,
        json!({
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY + DAS_PROFIT + OWNER_PROFIT,
            "type": {
                "code_hash": "{{sub-account-cell-type}}",
                "args": "das00002.bit"
            },
            "data": {
                "root": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "das_profit": DAS_PROFIT,
                "owner_profit": OWNER_PROFIT,
            }
        }),
    );
}

fn before_each() -> TemplateGenerator {
    let mut template = init("recycle_expired_account", None);

    template.push_contract_cell("sub-account-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    template
}

#[test]
fn test_account_recycle_without_sub_account() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD - ACCOUNT_EXPIRATION_AUCTION_CONFIRMATION_PERIOD - 1,
            },
            "witness": {
                "enable_sub_account": 0,
            }
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    test_tx(template.as_json());
}

#[test]
fn test_account_recycle_with_sub_account_and_refund_to_das() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT,
        OWNER,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    test_tx(template.as_json());
}

#[test]
fn test_account_recycle_with_sub_account_and_no_refunds_to_das() {
    let mut template = before_each();
    let das_profit = 6_100_000_000u64 - 1;

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_input_sub_account_cell(
        &mut template,
        json!({
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY + das_profit + OWNER_PROFIT,
            "type": {
                "code_hash": "{{sub-account-cell-type}}",
                "args": "das00002.bit"
            },
            "data": {
                "root": "0x0000000000000000000000000000000000000000000000000000000000000000",
                // Simulate the profit of DAS is less than 61CKB.
                "das_profit": das_profit,
                "owner_profit": OWNER_PROFIT,
            }
        }),
    );

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT,
        OWNER,
    );
    // Because the profit of DAS is less than 61CKB, there is no need to refund to DAS any more.

    test_tx(template.as_json());
}

#[test]
fn challenge_account_recycle_missing_sub_account() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    // Simulate forgetting to recycle the SubAccountCell at the same time.
    // push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT,
        OWNER,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure);
}

#[test]
fn challenge_account_recycle_with_wrong_sub_account() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_input_sub_account_cell(
        &mut template,
        json!({
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY,
            "type": {
                "code_hash": "{{sub-account-cell-type}}",
                // Simulate recycling a SubAccountCell of some other AccountCell.
                "args": "das00099.bit"
            },
            "data": {
                "root": "0x0000000000000000000000000000000000000000000000000000000000000000"
            }
        }),
    );

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT,
        OWNER,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellIdNotMatch);
}

#[test]
fn challenge_account_recycle_account_in_expiration_grace_period() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                // Simulate the AccountCell has not been expired.
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            },
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStillCanNotRecycle);
}

#[test]
fn challenge_account_recycle_account_in_expiration_auction_period() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                // Simulate the AccountCell has been expired, but still in expired account auction status.
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - ACCOUNT_EXPIRATION_AUCTION_PERIOD,
            },
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStillCanNotRecycle);
}

#[test]
fn challenge_account_recycle_status_locked() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                "expired_at": TIMESTAMP - DAY_SEC * 90,
            },
            "witness": {
                // Simulate the AccountCell is in some status which can not be recycled directly.
                "status": (AccountStatus::Selling as u8),
            }
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked);
}

#[test]
fn challenge_account_recycle_with_wrong_prev_account() {
    let mut template = before_each();

    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                // Simulate the previous AccountCell is not match with the expired AccountCell.
                "account": "das00099.bit",
                "next": "das00100.bit",
            },
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00002.bit",
                "next": "das00003.bit",
                "expired_at": TIMESTAMP - DAY_SEC * 90,
            },
        }),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00099.bit",
                "next": "das00100.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellMissingPrevAccount);
}

#[test]
fn challenge_account_recycle_update_next_error() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                // Simulate updating the next of the previous AccountCell to a wrong value.
                "next": "das00099.bit",
            },
        }),
    );
    push_output_balance_cell(&mut template, util::gen_account_cell_capacity(8), OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellNextUpdateError);
}

#[test]
fn challenge_account_recycle_refunds_owner() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT
            - ACCOUNT_OPERATE_FEE,
        OWNER_1,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    challenge_tx(template.as_json(), ErrorCode::ChangeError);
}

#[test]
fn challenge_account_recycle_owner_refunds_capacity() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8) +
            SUB_ACCOUNT_BASIC_CAPACITY +
            SUB_ACCOUNT_PREPARED_FEE_CAPACITY +
            OWNER_PROFIT -
            // Simulate the capacity of refunds to owner is not correct.
            ACCOUNT_OPERATE_FEE - 1,
        OWNER,
    );
    push_output_normal_cell(&mut template, DAS_PROFIT, DAS_WALLET_LOCK_ARGS);

    challenge_tx(template.as_json(), ErrorCode::ChangeError);
}

#[test]
fn challenge_account_recycle_das_refunds_capacity() {
    let mut template = before_each();

    push_prev_account_cell(&mut template);
    push_expired_account_cell(&mut template);
    push_expired_sub_account_cell(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "data": {
                "account": "das00001.bit",
                "next": "das00003.bit",
            },
        }),
    );
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8)
            + SUB_ACCOUNT_BASIC_CAPACITY
            + SUB_ACCOUNT_PREPARED_FEE_CAPACITY
            + OWNER_PROFIT
            - ACCOUNT_OPERATE_FEE,
        OWNER,
    );
    // Simulate the capacity of refunds to owner is not correct.
    push_output_normal_cell(&mut template, DAS_PROFIT - 1, DAS_WALLET_LOCK_ARGS);

    challenge_tx(template.as_json(), ErrorCode::ChangeError);
}
