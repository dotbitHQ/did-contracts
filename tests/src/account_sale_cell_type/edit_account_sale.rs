use super::common::*;
use crate::util::{accounts::*, constants::*, error::Error, template_generator::*, template_parser::*};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("edit_account_sale", Some("0x00"));

    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    template
}

#[test]
fn test_account_sale_edit() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE + 10_000_000_000
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_edit_old_version() {
    let mut template = init("edit_account_sale", Some("0x00"));

    // inputs
    push_input_account_sale_cell_v1(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE + 10_000_000_000
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_sale_edit_with_manager() {
    // Simulate send the transaction as manager.
    let mut template = init("edit_account_sale", Some("0x01"));

    // inputs
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellPermissionDenied)
}

#[test]
fn challenge_account_sale_edit_lock_consistent() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                // Simulate modifying the owner lock of the AccountCell.
                "owner_lock_args": "0x051111000000000000000000000000000000001111",
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_sale_edit_account_consistent() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the account has been modified accidentally.
                "account": "zzzzz.bit",
                "price": PRICE
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_edit_account_id_consistent() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the account ID is mismatched with the account.
                "account_id": "0x1111000000000000000000000000000000001111",
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_edit_started_at_consistent() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE,
                // Simulate the started_at field has been modified accidentally.
                "started_at": TIMESTAMP - 1
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellStartedAtInvalid)
}

#[test]
fn challenge_account_sale_edit_fee_spent() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            // Simulate too much fee has been spent.
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE - 1,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::TxFeeSpentError)
}

#[test]
fn challenge_account_sale_edit_fee_empty() {
    let mut template = init("edit_account_sale", Some("0x00"));

    push_input_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            // Simulate spend basic capacity as fee.
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY - 1,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::TxFeeSpentError)
}

#[test]
fn challenge_account_sale_edit_price() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                // Simulate modify the price to lower than the minimum requirement.
                "price": ACCOUNT_SALE_MIN_PRICE - 1,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellPriceTooSmall)
}

#[test]
fn challenge_account_sale_edit_no_change() {
    let mut template = before_each();

    // outputs
    push_output_account_sale_cell(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate neither price nor description is changed.
                "account": ACCOUNT
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_sale_edit_keep_old_version() {
    let mut template = init("edit_account_sale", Some("0x00"));

    // inputs
    push_input_account_sale_cell_v1(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );

    // outputs
    // Simulate keeping the AccountSaleCell as the old version.
    push_output_account_sale_cell_v1(
        &mut template,
        json!({
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                "price": PRICE + 10_000_000_000
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}
