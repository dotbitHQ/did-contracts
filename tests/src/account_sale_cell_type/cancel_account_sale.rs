use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::*, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::*;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT_1,
                "price": PRICE
            }
        }),
    );

    template
}

#[test]
fn test_account_sale_cancel() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    test_tx(template.as_json());
}

#[test]
fn test_account_sale_cancel_old_version() {
    let mut template = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell_v1(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT_1,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_sale_cancel_with_manager() {
    // Simulate send the transaction as manager.
    let mut template = init("cancel_account_sale", Some("0x01"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellPermissionDenied)
}

#[test]
fn challenge_account_sale_cancel_account_consistent() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the owner lock of AccountCell has been modified accidentally.
                "owner_lock_args": "0x051111000000000000000000000000000000001111",
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), ErrorCode::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_sale_cancel_account_expired() {
    let mut template = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1,
                // Simulate the AccountCell has been expired when user trying to sell it.
                "expired_at": (TIMESTAMP - 1),
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT_1,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1,
                "expired_at": (TIMESTAMP - 1),
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellInExpirationGracePeriod,
    )
}

#[test]
fn challenge_account_sale_cancel_account_input_status() {
    let mut template = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                // Simulate the AccountCell in inputs has been in normal status.
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                "account": ACCOUNT_1,
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_cancel_account_output_status() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked)
}

#[test]
fn challenge_account_sale_cancel_sale_account_mismatch() {
    let mut template = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "witness": {
                // Simulate the AccountSaleCell do not have the same account name as the AccountCell.
                "account": "zzzzz.bit",
                "price": PRICE
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        SELLER,
    );

    challenge_tx(template.as_json(), ErrorCode::AccountSaleCellAccountIdInvalid)
}

#[test]
fn challenge_account_sale_cancel_change_owner() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE,
        "0x051111000000000000000000000000000000001111",
    );

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_account_sale_cancel_change_capacity() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT_1
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_output_balance_cell(
        &mut template,
        ACCOUNT_SALE_BASIC_CAPACITY + ACCOUNT_SALE_PREPARED_FEE_CAPACITY - SECONDARY_MARKET_COMMON_FEE - 1,
        SELLER,
    );

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}
