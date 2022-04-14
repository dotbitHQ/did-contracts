use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> (TemplateGenerator, u64) {
    let mut template = init("start_account_sale", Some("0x00"));

    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    let total_input = 600_000_000_000;
    push_input_balance_cell(&mut template, total_input / 3, SELLER);
    push_input_balance_cell(&mut template, total_input / 3, SELLER);
    push_input_balance_cell(&mut template, total_input / 3, SELLER);

    (template, total_input)
}

fn push_common_outputs(template: &mut TemplateGenerator, total_input: u64) {
    push_output_account_cell(
        template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        template,
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
    push_output_balance_cell(
        template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );
}

#[test]
fn test_account_sale_start() {
    let (mut template, total_input) = before_each();

    push_common_outputs(&mut template, total_input);

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_start_with_lock_upgrade() {
    let mut template = init("start_account_sale", Some("0x00"));

    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate upgrading the type of the owner lock during this transaction.
                "owner_lock_args": "0x030000000000000000000000000000000000001111",
                "manager_lock_args": "0x030000000000000000000000000000000000001111"
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_balance_cell(&mut template, 600_000_000_000, SELLER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
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
    push_output_balance_cell(
        &mut template,
        600_000_000_000 - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY,
        SELLER,
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_sale_start_with_manager() {
    // Simulate send the transaction as manager.
    let mut template = init("start_account_sale", Some("0x01"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_balance_cell(&mut template, 600_000_000_000, SELLER);

    // outputs
    push_common_outputs(&mut template, 600_000_000_000);

    challenge_tx(template.as_json(), Error::AccountCellPermissionDenied)
}

#[test]
fn challenge_account_sale_start_account_consistent() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the owner lock of AccountCell has been modified accidentally.
                "owner_lock_args": "0x051111000000000000000000000000000000001111",
                "manager_lock_args": "0x051111000000000000000000000000000000001111"
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
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
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_sale_start_account_expired() {
    let mut template = init("start_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
                // Simulate the AccountCell has been expired when user trying to sell it.
                "expired_at": (TIMESTAMP - 1),
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_balance_cell(&mut template, 600_000_000_000, SELLER);

    // outputs
    push_common_outputs(&mut template, 600_000_000_000);

    challenge_tx(template.as_json(), Error::AccountCellInExpirationGracePeriod)
}

#[test]
fn challenge_account_sale_start_account_input_status() {
    let mut template = init("start_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                // Simulate the AccountCell in inputs has been in selling status.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_balance_cell(&mut template, 600_000_000_000, SELLER);

    // outputs
    push_common_outputs(&mut template, 600_000_000_000);

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_start_account_output_status() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                // Simulate the AccountCell has been modified to wrong status accidentally.
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_account_sale_cell(
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
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_start_sale_capacity() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            // Simulate the AccountSaleCell do not get enough capacity.
            "capacity": ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - 1,
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
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE
            + 1,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellCapacityError)
}

#[test]
fn challenge_account_sale_start_sale_account() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the AccountSaleCell do not have the same account name as the AccountCell.
                "account": "yyyyy.bit",
                "price": PRICE
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_start_sale_account_id() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the AccountSaleCell do not have the same account ID as the AccountCell.
                "account_id": "0x1111000000000000000000000000000000001111",
                "account": ACCOUNT,
                "price": PRICE
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_start_sale_price() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                // Simulate the AccountSaleCell's price is less than the minimum requirement.
                "price": "19_000_000_000"
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellPriceTooSmall)
}

#[test]
fn challenge_account_sale_start_sale_started_at() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
                // Simulate the AccountSaleCell do not have the same timestamp as which in the TimeCell.
                "started_at": TIMESTAMP - 1
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::AccountSaleCellStartedAtInvalid)
}

#[test]
fn challenge_account_sale_start_change_owner() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        // Simulate transfer changes to another lock.
        "0x052222000000000000000000000000000000002222",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_start_change_capacity() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT,
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        // Simulate transfer changes less than the user should get.
        total_input
            - ACCOUNT_SALE_BASIC_CAPACITY
            - ACCOUNT_SALE_PREPARED_FEE_CAPACITY
            - SECONDARY_MARKET_COMMON_FEE
            - 1,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_sale_start_with_old_version() {
    let (mut template, total_input) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    // Simulate creating the old version of AccountSaleCell.
    push_output_account_sale_cell_v1(
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
    push_output_balance_cell(
        &mut template,
        total_input - ACCOUNT_SALE_BASIC_CAPACITY - ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}
