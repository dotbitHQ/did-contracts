use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
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

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": timestamp
            }
        }),
    );
    push_input_balance_cell(&mut template, 1_000_000_000_000, OWNER);

    (template, timestamp)
}

#[test]
fn test_account_renew_not_create_income_cell() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    test_tx(template.as_json());
}

#[test]
fn test_account_renew_create_income_cell() {
    let (mut template, timestamp) = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": timestamp
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
                "expired_at": timestamp + 31_536_000,
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
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the owner of the AccountCell was changed.
                "owner_lock_args": "0x000000000000000000000000000000000000003333",
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(template.as_json(), Error::CellLockCanNotBeModified)
}

#[test]
fn challenge_account_renew_less_than_one_year() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                // Simulate the increment of the expired_at is less than one year.
                "expired_at": timestamp + 31_536_000 - 1,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000, OWNER);

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationMustLongerThanYear)
}

#[test]
fn challenge_account_renew_payment_less_than_one_year() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
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

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationMustLongerThanYear)
}

#[test]
fn challenge_account_renew_payment_less_than_increment() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": timestamp + 31_536_000 * 3,
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

    challenge_tx(template.as_json(), Error::AccountCellRenewDurationBiggerThanPayed)
}

#[test]
fn challenge_account_renew_change_amount() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 500_000_000_000 - 1, OWNER);

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_renew_change_owner() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER,
            },
            "data": {
                "expired_at": timestamp + 31_536_000,
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(
        &mut template,
        500_000_000_000,
        "0x000000000000000000000000000000000000003333",
    );

    challenge_tx(template.as_json(), Error::ChangeError)
}

#[test]
fn challenge_account_renew_income_cell_capacity() {
    let (mut template, timestamp) = init_for_renew("renew_account", None);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": OWNER
            },
            "data": {
                "expired_at": timestamp
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
                "expired_at": timestamp + 31_536_000,
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

    challenge_tx(template.as_json(), Error::IncomeCellCapacityError)
}
