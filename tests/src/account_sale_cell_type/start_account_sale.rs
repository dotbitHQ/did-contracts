use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, owner: &str, timestamp: u64) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, owner: &str, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_account_sale_cell(template: &mut TemplateGenerator, owner: &str, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": (ACCOUNT_SALE_CELL_BASIC_CAPACITY + ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY).to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": owner
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "xxxxx.bit",
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        None,
    );
}

fn before_each() -> (TemplateGenerator, u64, &'static str) {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
    let owner = "0x050000000000000000000000000000000000001111";

    push_input_account_cell(&mut template, owner, timestamp);
    push_input_balance_cell(&mut template, 40_000_000_000, owner);

    (template, timestamp, owner)
}

test_with_generator!(test_account_sale_start, || {
    let (mut template, timestamp, owner) = before_each();

    push_output_account_cell(&mut template, owner, timestamp);
    push_output_account_sale_cell(&mut template, owner, timestamp);
    push_output_balance_cell(
        &mut template,
        40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
        owner,
    );

    template.as_json()
});

test_with_generator!(test_account_sale_start_with_lock_upgrade, || {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
    let owner = "0x050000000000000000000000000000000000001111";

    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                // Simulate upgrading the type of the owner lock during this transaction.
                "owner_lock_args": "0x030000000000000000000000000000000000001111",
                "manager_lock_args": "0x030000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

    push_input_balance_cell(&mut template, 40_000_000_000, owner);

    // outputs
    push_output_account_cell(&mut template, owner, timestamp);
    push_output_account_sale_cell(&mut template, owner, timestamp);
    push_output_balance_cell(
        &mut template,
        40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
        owner,
    );

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_start_with_manager,
    Error::AccountCellPermissionDenied,
    || {
        // Simulate send the transaction as manager.
        let (mut template, timestamp) = init("start_account_sale", Some("0x01"));
        let owner = "0x050000000000000000000000000000000000001111";

        // inputs
        push_input_account_cell(&mut template, owner, timestamp);
        push_input_balance_cell(&mut template, 40_000_000_000, owner);

        // outputs
        push_output_account_cell(&mut template, owner, timestamp);
        push_output_account_sale_cell(&mut template, owner, timestamp);
        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_consistent,
    Error::CellLockCanNotBeModified,
    || {
        let (mut template, timestamp, owner) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    // Simulate the owner lock of AccountCell has been modified accidentally.
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
                    "manager_lock_args": owner
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );

        push_output_account_sale_cell(&mut template, owner, timestamp);
        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_expired,
    Error::AccountCellInExpirationGracePeriod,
    || {
        let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
        let owner = "0x050000000000000000000000000000000000001111";

        // inputs
        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": owner,
                    "manager_lock_args": owner
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    // Simulate the AccountCell has been expired when user trying to sell it.
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - YEAR_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );
        template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

        push_input_balance_cell(&mut template, 40_000_000_000, owner);

        // outputs
        push_output_account_cell(&mut template, owner, timestamp);
        push_output_account_sale_cell(&mut template, owner, timestamp);
        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_input_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
        let owner = "0x050000000000000000000000000000000000001111";

        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell in inputs has been in selling status.
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );
        template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

        push_input_balance_cell(&mut template, 40_000_000_000, owner);

        // outputs
        push_output_account_cell(&mut template, owner, timestamp);
        push_output_account_sale_cell(&mut template, owner, timestamp);
        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_output_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp, owner) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": owner,
                    "manager_lock_args": owner
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell has been modified to wrong status accidentally.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_account_sale_cell(&mut template, owner, timestamp);
        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_capacity,
    Error::AccountSaleCellCapacityError,
    || {
        let (mut template, timestamp, owner) = before_each();

        push_output_account_cell(&mut template, owner, timestamp);

        template.push_output(
            json!({
                // Simulate the AccountSaleCell do not get enough capacity.
                "capacity": "20_099_999_999",
                "lock": {
                    "owner_lock_args": owner,
                    "manager_lock_args": owner
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_account,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp, owner) = before_each();

        push_output_account_cell(&mut template, owner, timestamp);

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
                    // Simulate the AccountSaleCell do not have the same account name as the AccountCell.
                    "account": "zzzzz.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_account_id,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp, owner) = before_each();

        push_output_account_cell(&mut template, owner, timestamp);

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
                    // Simulate the AccountSaleCell do not have the same account ID as the AccountCell.
                    "account_id": "0x1111000000000000000000000000000000001111",
                    "account": "xxxxx.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_price,
    Error::AccountSaleCellPriceTooSmall,
    || {
        let (mut template, timestamp, owner) = before_each();

        push_output_account_cell(&mut template, owner, timestamp);

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
                    "account": "xxxxx.bit",
                    // Simulate the AccountSaleCell's price is less than the minimum requirement.
                    "price": "19_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_started_at,
    Error::AccountSaleCellStartedAtInvalid,
    || {
        let (mut template, timestamp, owner) = before_each();

        push_output_account_cell(&mut template, owner, timestamp);

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
                    "account": "xxxxx.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    // Simulate the AccountSaleCell do not have the same timestamp as which in the TimeCell.
                    "started_at": timestamp - 1
                }
            }),
            None,
        );

        push_output_balance_cell(
            &mut template,
            40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
            owner,
        );

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_sale_start_change_owner, Error::ChangeError, || {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
    let owner = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_account_cell(&mut template, owner, timestamp);
    push_input_balance_cell(&mut template, 40_000_000_000, owner);

    // outputs
    push_output_account_cell(&mut template, owner, timestamp);
    push_output_account_sale_cell(&mut template, owner, timestamp);
    push_output_balance_cell(
        &mut template,
        40_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
        // Simulate transfer changes to another lock.
        "0x050000000000000000000000000000000000002222",
    );

    template.as_json()
});

challenge_with_generator!(challenge_account_sale_start_change_capacity, Error::ChangeError, || {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));
    let owner = "0x050000000000000000000000000000000000001111";

    // inputs
    push_input_account_cell(&mut template, owner, timestamp);
    push_input_balance_cell(&mut template, 40_000_000_000, owner);

    // outputs
    push_output_account_cell(&mut template, owner, timestamp);
    push_output_account_sale_cell(&mut template, owner, timestamp);
    push_output_balance_cell(
        &mut template,
        // Simulate transfer changes less than the user should get.
        39_000_000_000 - ACCOUNT_SALE_CELL_BASIC_CAPACITY - ACCOUNT_SALE_CELL_PREPARED_FEE_CAPACITY,
        owner,
    );

    template.as_json()
});
