use super::common::init;
use crate::util::constants::{DAY_SEC, MONTH_SEC, YEAR_SEC};
use crate::util::{self, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
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

fn push_input_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64, owner: &str, manager: &str) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "das00001.bit",
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp - MONTH_SEC
            }
        }),
        None,
    );
}

fn push_output_account_cell(
    template: &mut TemplateGenerator,
    timestamp: u64,
    owner: &str,
    manager: &str,
    status: AccountStatus,
) {
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
                "status": (status as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_balance_cell(template: &mut TemplateGenerator, owner: &str, manager: &str, capacity: u64) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager,
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}

fn before_each(owner: &str, manager: &str) -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("force_recover_account_status", None);

    template.push_contract_cell("account-sale-cell-type", false);
    template.push_contract_cell("balance-cell-type", false);

    push_input_account_cell(&mut template, timestamp, owner, manager, AccountStatus::Selling);
    push_input_account_sale_cell(&mut template, timestamp, owner, manager);

    (template, timestamp)
}

test_with_generator!(test_account_force_recover_account_status, || {
    let owner = "0x000000000000000000000000000000000000001111";
    let manager = "0x000000000000000000000000000000000000001111";
    let (mut template, timestamp) = before_each(owner, manager);

    push_output_account_cell(&mut template, timestamp, owner, manager, AccountStatus::Normal);
    push_output_balance_cell(&mut template, owner, manager, 20_099_990_000);

    template.as_json()
});
