use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, timestamp: u64) {
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
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_input_fee_cell(template: &mut TemplateGenerator) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111",
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_output(
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
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));

    push_input_account_cell(&mut template, timestamp);
    push_input_fee_cell(&mut template);

    (template, timestamp)
}

test_with_generator!(test_account_sale_start, || {
    let (mut template, timestamp) = before_each();

    push_output_account_cell(&mut template, timestamp);
    push_output_account_sale_cell(&mut template, timestamp);

    template.as_json()
});

test_with_generator!(test_account_sale_start_with_lock_upgrade, || {
    let (mut template, timestamp) = init("start_account_sale", Some("0x00"));

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

    push_input_fee_cell(&mut template);

    // outputs
    push_output_account_cell(&mut template, timestamp);
    push_output_account_sale_cell(&mut template, timestamp);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_start_with_manager,
    Error::AccountCellPermissionDenied,
    || {
        let (mut template, timestamp) = init("start_account_sale", Some("0x01"));

        // inputs
        push_input_account_cell(&mut template, timestamp);
        push_input_fee_cell(&mut template);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_consistent,
    Error::CellLockCanNotBeModified,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    // Simulate the owner lock of AccountCell has been modified accidentally.
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
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
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );

        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_expired,
    Error::AccountCellInExpirationGracePeriod,
    || {
        let (mut template, timestamp) = init("start_account_sale", Some("0x00"));

        // inputs
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

        push_input_fee_cell(&mut template);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_input_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp) = init("start_account_sale", Some("0x00"));

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

        push_input_fee_cell(&mut template);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_account_output_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
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
                    // Simulate the AccountCell has been modified to wrong status accidentally.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_capacity,
    Error::AccountSaleCellCapacityError,
    || {
        let (mut template, timestamp) = before_each();

        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                // Simulate the AccountSaleCell do not get enough capacity.
                "capacity": "20_099_999_999",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_account,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp) = before_each();

        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_account_id,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp) = before_each();

        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_price,
    Error::AccountSaleCellPriceTooSmall,
    || {
        let (mut template, timestamp) = before_each();

        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_start_sale_started_at,
    Error::AccountSaleCellStartedAtInvalid,
    || {
        let (mut template, timestamp) = before_each();

        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);
