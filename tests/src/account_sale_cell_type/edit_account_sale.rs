use super::common::*;
use crate::util::{error::Error, template_generator::*, template_parser::*};
use serde_json::json;

fn push_input_account_sale_cell(
    template: &mut TemplateGenerator,
    owner: &str,
    account: &str,
    price: u64,
    timestamp: u64,
) {
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
                "account": account,
                "price": price.to_string(),
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_sale_cell(
    template: &mut TemplateGenerator,
    owner: &str,
    account: &str,
    price: u64,
    timestamp: u64,
) {
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        Some(2),
    );
}

fn before_each() -> (TemplateGenerator, u64, &'static str, &'static str) {
    let (mut template, timestamp) = init("edit_account_sale", Some("0x00"));
    let owner = "0x050000000000000000000000000000000000001111";
    let account = "xxxxx.bit";

    push_input_account_sale_cell(&mut template, owner, account, 200_000_000_000, timestamp);

    (template, timestamp, owner, account)
}

#[test]
fn test_account_sale_edit() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    push_output_account_sale_cell(&mut template, owner, account, 400_000_000_000, timestamp);

    test_tx(template.as_json());
}

#[test]
fn challenge_account_sale_edit_with_manager() {
    let (mut template, timestamp) = init("edit_account_sale", Some("0x01"));
    let owner = "0x050000000000000000000000000000000000001111";
    let account = "xxxxx.bit";

    // inputs
    push_input_account_sale_cell(&mut template, owner, account, 200_000_000_000, timestamp);

    // outputs
    push_output_account_sale_cell(&mut template, owner, account, 400_000_000_000, timestamp);

    challenge_tx(template.as_json(), Error::AccountCellPermissionDenied)
}

#[test]
fn challenge_account_sale_edit_lock_consistent() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                // Simulate the owner lock has been modified accidentally.
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_sale_edit_account_consistent() {
    let (mut template, timestamp, owner, _) = before_each();

    // outputs
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                // Simulate the account has been modified accidentally.
                "account": "zzzzz.bit",
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_edit_account_id_consistent() {
    let (mut template, timestamp, owner, _) = before_each();

    // outputs
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                // Simulate the account ID has been modified accidentally.
                "account_id": "0x1111000000000000000000000000000000001111",
                "account": "xxxxx.bit",
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_edit_started_at_consistent() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": "40_000_000_000",
                "description": "This is another account description.",
                // Simulate the started_at field has been modified accidentally.
                "started_at": timestamp - 1
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellStartedAtInvalid)
}

#[test]
fn challenge_account_sale_edit_fee_spent() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
        json!({
            // Simulate too much fee has been spent.
            "capacity": "20_099_980_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellFeeError)
}

#[test]
fn challenge_account_sale_edit_fee_empty() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
        json!({
            // Simulate spend basic capacity as fee.
            "capacity": "19_999_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellFeeError)
}

#[test]
fn challenge_account_sale_edit_price() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": account,
                // Simulate modify the price to lower than the minimum requirement.
                "price": "19_900_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellPriceTooSmall)
}

#[test]
fn challenge_account_sale_edit_no_change() {
    let (mut template, timestamp, owner, account) = before_each();

    // outputs
    template.push_output(
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
                "account": account,
                // Simulate neither price nor description is changed.
                "price": "200_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}
