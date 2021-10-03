use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

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

fn push_output_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": "20_099_990_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "xxxxx.bit",
                "price": "40_000_000_000",
                "description": "This is another account description.",
                "started_at": timestamp
            }
        }),
        None,
    );
}

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("edit_account_sale", Some("0x00"));

    push_input_account_sale_cell(&mut template, timestamp);

    (template, timestamp)
}

test_with_generator!(test_account_sale_edit, || {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_sale_cell(&mut template, timestamp);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_edit_with_manager,
    Error::AccountCellPermissionDenied,
    || {
        let (mut template, timestamp) = init("edit_account_sale", Some("0x01"));

        // inputs
        push_input_account_sale_cell(&mut template, timestamp);

        // outputs
        push_output_account_sale_cell(&mut template, timestamp);

        template.as_json()
    }
);
challenge_with_generator!(
    challenge_account_sale_edit_lock_consistent,
    Error::InvalidTransactionStructure,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    // Simulate the owner lock has been modified accidentally.
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "price": "40_000_000_000",
                    "description": "This is another account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_account_consistent,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_account_id_consistent,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
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

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_started_at_consistent,
    Error::AccountSaleCellStartedAtInvalid,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "price": "40_000_000_000",
                    "description": "This is another account description.",
                    // Simulate the started_at field has been modified accidentally.
                    "started_at": timestamp - 1
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_fee_spent,
    Error::AccountSaleCellFeeError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                // Simulate too much fee has been spent.
                "capacity": "20_099_980_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "price": "40_000_000_000",
                    "description": "This is another account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_fee_empty,
    Error::AccountSaleCellFeeError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                // Simulate spend basic capacity as fee.
                "capacity": "19_999_990_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "price": "40_000_000_000",
                    "description": "This is another account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_edit_price,
    Error::AccountSaleCellPriceTooSmall,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": "20_099_990_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    "account": "xxxxx.bit",
                    // Simulate modify the price to lower than the minimum requirement.
                    "price": "19_900_000_000",
                    "description": "This is another account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );

        template.as_json()
    }
);
