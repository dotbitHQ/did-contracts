use super::common::init;
use crate::util::{
    self, accounts::*, constants::*, error::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn push_input_account_sale_cell(template: &mut TemplateGenerator) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": ACCOUNT_1,
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": TIMESTAMP - MONTH_SEC,
                "buyer_inviter_profit_rate": SALE_BUYER_INVITER_PROFIT_RATE
            }
        }),
        Some(2),
    );
}

fn before_each() -> TemplateGenerator {
    let mut template = init("force_recover_account_status", None);

    template.push_contract_cell("account-sale-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(&mut template);

    template
}

#[test]
fn test_account_force_recover_account_status() {
    let mut template = before_each();

    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(&mut template, 20_099_990_000, OWNER);

    test_tx(template.as_json());
}

#[test]
fn challenge_account_force_recover_account_still_ok() {
    let mut template = init("force_recover_account_status", None);

    template.push_contract_cell("account-sale-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                // Simulate the AccountCell is still available.
                "expired_at": TIMESTAMP,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(&mut template);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                "expired_at": TIMESTAMP,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(&mut template, 20_099_990_000, OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellIsNotExpired);
}

#[test]
fn challenge_account_force_recover_account_in_expiration_grace_period() {
    let mut template = init("force_recover_account_status", None);

    template.push_contract_cell("account-sale-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                // Simulate the AccountCell is still available.
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(&mut template);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "data": {
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(&mut template, 20_099_990_000, OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellIsNotExpired);
}
