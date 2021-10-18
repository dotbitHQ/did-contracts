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
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Some(2),
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_input_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_input(
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
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_refund_cell(template: &mut TemplateGenerator) {
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
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
}

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("cancel_account_sale", Some("0x00"));

    // inputs
    push_input_account_cell(&mut template, timestamp);
    push_input_account_sale_cell(&mut template, timestamp);

    (template, timestamp)
}

test_with_generator!(test_account_sale_cancel, || {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(&mut template, timestamp);
    push_output_refund_cell(&mut template);

    template.as_json()
});

fn after_test_outputs(template: &mut TemplateGenerator, _timestamp: u64) -> serde_json::Value {
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
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

    template.as_json()
}

challenge_with_generator!(
    challenge_account_sale_cancel_with_manager,
    Error::AccountCellPermissionDenied,
    || {
        let (mut template, timestamp) = init("cancel_account_sale", Some("0x01"));

        // inputs
        push_input_account_cell(&mut template, timestamp);
        push_input_account_sale_cell(&mut template, timestamp);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_account_consistent,
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
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_account_expired,
    Error::AccountCellInExpirationGracePeriod,
    || {
        let (mut template, timestamp) = init("cancel_account_sale", Some("0x00"));

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
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );
        template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

        push_input_account_sale_cell(&mut template, timestamp);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_account_input_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp) = init("cancel_account_sale", Some("0x00"));

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
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell in inputs has been in normal status.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );
        template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

        push_input_account_sale_cell(&mut template, timestamp);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_account_output_status,
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
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );

        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_sale_account,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp) = init("cancel_account_sale", Some("0x00"));

        // inputs
        push_input_account_cell(&mut template, timestamp);

        template.push_input(
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
        template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_refund_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_refund_lock,
    Error::AccountSaleCellRefundError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    // Simulate refundind with wrong lock script.
                    "owner_lock_args": "0x030000000000000000000000000000000000001111",
                    "manager_lock_args": "0x030000000000000000000000000000000000001111",
                },
                "type": {
                    "code_hash": "{{balance-cell-type}}"
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_cancel_refund_capacity,
    Error::AccountSaleCellRefundError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        push_output_account_cell(&mut template, timestamp);

        template.push_output(
            json!({
                // Simulate refundind with wrong capacity.
                "capacity": "20_099_980_000",
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

        template.as_json()
    }
);
