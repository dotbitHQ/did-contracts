use super::common::init;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types::constants::AccountStatus;
use serde_json::json;

fn push_input_account_cell(
    template: &mut TemplateGenerator,
    timestamp: u64,
    owner: &str,
    manager: &str,
    status: AccountStatus,
) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": timestamp - DAY_SEC,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": timestamp - YEAR_SEC,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (status as u8)
            }
        }),
        Some(2),
    );
    template.push_empty_witness();
}

fn push_input_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64, owner: &str) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "das00001.bit",
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp - MONTH_SEC,
                "buyer_inviter_profit_rate": SALE_BUYER_INVITER_PROFIT_RATE
            }
        }),
        None,
    );
}

fn before_each() -> (TemplateGenerator, u64, &'static str, &'static str) {
    let (mut template, timestamp) = init("force_recover_account_status", None);
    let owner = "0x000000000000000000000000000000000000001111";
    let manager = "0x000000000000000000000000000000000000001111";

    template.push_contract_cell("account-sale-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    push_input_account_cell(&mut template, timestamp, owner, manager, AccountStatus::Selling);
    push_input_account_sale_cell(&mut template, timestamp, owner);

    (template, timestamp, owner, manager)
}

#[test]
fn test_account_force_recover_account_status() {
    let (mut template, timestamp, owner, manager) = before_each();

    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "das00001.bit",
                "next": "das00014.bit",
                "expired_at": timestamp - DAY_SEC,
            },
            "witness": {
                "account": "das00001.bit",
                "registered_at": timestamp - YEAR_SEC,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );

    push_output_balance_cell(&mut template, 20_099_990_000, owner);

    test_tx(template.as_json());
}
