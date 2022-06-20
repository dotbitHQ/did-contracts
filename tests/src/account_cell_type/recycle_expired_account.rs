use super::common::init;
use crate::util::{
    self, error::Error, accounts::*, constants::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

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
                "expired_at": TIMESTAMP - DAY_SEC * 90,
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
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY,
            "type": {
                "code_hash": "{{sub-account-cell-type}}",
                "args": "das00002.bit"
            },
            "data": {
                "root": "0x0000000000000000000000000000000000000000000000000000000000000000"
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
                "expired_at": TIMESTAMP - DAY_SEC * 90,
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
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    test_tx(template.as_json());
}

#[test]
fn test_account_recycle_with_sub_account() {
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
        util::gen_account_cell_capacity(8) + SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY,
        OWNER,
    );

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
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
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
        util::gen_account_cell_capacity(8) + SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY,
        OWNER,
    );

    challenge_tx(template.as_json(), Error::AccountCellIdNotMatch);
}

#[test]
fn challenge_account_recycle_account_not_expired() {
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
                "expired_at": TIMESTAMP,
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
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    challenge_tx(template.as_json(), Error::AccountCellHasNotExpired);
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
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked);
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
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    challenge_tx(template.as_json(), Error::AccountCellMissingPrevAccount);
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
    push_output_balance_cell(
        &mut template,
        util::gen_account_cell_capacity(8),
        OWNER,
    );

    challenge_tx(template.as_json(), Error::AccountCellNextUpdateError);
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
        util::gen_account_cell_capacity(8) +
            SUB_ACCOUNT_BASIC_CAPACITY +
            SUB_ACCOUNT_PREPARED_FEE_CAPACITY -
            ACCOUNT_OPERATE_FEE,
        // Simulate the refund is sent to another user.
        OWNER_1,
    );

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_account_recycle_refunds_capacity() {
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
            SUB_ACCOUNT_PREPARED_FEE_CAPACITY -
            // Simulate the capacity of refunds is not correct.
            ACCOUNT_OPERATE_FEE - 1,
        OWNER,
    );

    challenge_tx(template.as_json(), Error::ChangeError);
}
