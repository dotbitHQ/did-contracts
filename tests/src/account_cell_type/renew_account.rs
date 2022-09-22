use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::*, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn push_simple_output_income_cell(template: &mut TemplateGenerator) {
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "500_000_000_000"
                    }
                ]
            }
        }),
    );
}

fn before_each() -> TemplateGenerator {
    let mut template = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);

    template
}

#[test]
fn test_account_renew_not_create_income_cell() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    test_tx(template.as_json());
}

#[test]
fn test_account_renew_create_income_cell() {
    let mut template = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            }
        }),
    );
    push_input_balance_cell(&mut template, 500_000_000_000, OWNER);
    push_input_balance_cell(
        &mut template,
        20_000_000_000,
        "0x0000000000000000000000000000000000000000",
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    // Simulate creating IncomeCell by sender.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "500_000_000_000"
                    }
                ]
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_renew_modify_owner() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the owner of the AccountCell was changed.
                "owner_lock_args": "0x000000000000000000000000000000000000003333",
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(template.as_json(), ErrorCode::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_renew_less_than_one_year() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                // Simulate the increment of the expired_at is less than one year.
                "expired_at": TIMESTAMP + 31_536_000 - 1,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellRenewDurationMustLongerThanYear,
    )
}

#[test]
fn challenge_account_renew_payment_less_than_one_year() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        // Simulate a payment shortfall.
                        "capacity": (500_000_000_000u64 - 1).to_string()
                    }
                ]
            }
        }),
    );
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellRenewDurationMustLongerThanYear,
    )
}

#[test]
fn challenge_account_renew_payment_less_than_increment() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000 * 3,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        // Simulate a payment shortfall.
                        "capacity": 500_000_000_000u64.to_string()
                    }
                ]
            }
        }),
    );
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellRenewDurationBiggerThanPayed,
    )
}

#[test]
fn challenge_account_renew_change_amount() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000 - 1, OWNER);

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_account_renew_change_owner() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        500_000_000_000,
        "0x000000000000000000000000000000000000003333",
    );

    challenge_tx(template.as_json(), ErrorCode::ChangeError)
}

#[test]
fn challenge_account_renew_income_cell_capacity() {
    let mut template = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            }
        }),
    );
    push_input_balance_cell(&mut template, 500_000_000_000, OWNER);
    push_input_balance_cell(
        &mut template,
        20_000_000_000,
        "0x0000000000000000000000000000000000000000",
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            }
        }),
    );
    push_output_income_cell(
        &mut template,
        json!({
            // Simulate wrong capacity for the IncomeCell.
            "capacity": 20_100_000_000u64,
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "500_000_000_000"
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), ErrorCode::IncomeCellCapacityError)
}

#[test]
fn challenge_account_renew_locked_for_cross_chain() {
    let mut template = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": TIMESTAMP
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": TIMESTAMP + 31_536_000,
            },
            "witness": {
                "status": (AccountStatus::LockedForCrossChain as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked)
}

#[test]
fn challenge_account_renew_expired_account() {
    let mut template = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                // Simulate the owner of the AccountCell was changed.
                "expired_at": TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                "expired_at": (TIMESTAMP - ACCOUNT_EXPIRATION_GRACE_PERIOD - 1) + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellHasExpired)
}
