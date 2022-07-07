use super::common::init;
use crate::util::{
    self, accounts::*, constants::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, status: AccountStatus) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": TIMESTAMP - DAY_SEC,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": TIMESTAMP - YEAR_SEC,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (status as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

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
                "account": "das00001.bit",
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

    push_input_account_cell(&mut template, AccountStatus::Selling);
    push_input_account_sale_cell(&mut template);

    template
}

#[test]
fn test_account_force_recover_account_status() {
    let mut template = before_each();

    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": MANAGER
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": TIMESTAMP - DAY_SEC,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": TIMESTAMP - YEAR_SEC,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(3),
    );

    push_output_balance_cell(&mut template, 20_099_990_000, OWNER);

    test_tx(template.as_json());
}
