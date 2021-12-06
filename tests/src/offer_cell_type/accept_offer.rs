use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_simple_output_income_cell(template: &mut TemplateGenerator) {
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    // It is a conversion in this transaction that the first record always belong to the creator of the IncomeCell.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "2_000_000_000"
                    }
                ]
            }
        }),
    );
}

fn push_common_outputs(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(template);
    push_output_balance_cell(template, 194_000_000_000, SELLER);
}

fn before_each() -> TemplateGenerator {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
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
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    // Transaction builder's BalanceCell
    push_input_balance_cell(
        &mut template,
        100_000_000_000,
        "0x050000000000000000000000000000000000003333",
    );

    template
}

#[test]
fn test_offer_accept_offer() {
    let mut template = before_each();

    // outputs
    push_common_outputs(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_offer_accept_offer_account_expired() {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SELLER,
                "manager_lock_args": SELLER
            },
            "data": {
                "account": ACCOUNT,
                // Simulate the AccountCell has been expired.
                "expired_at": TIMESTAMP - 1,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::AccountCellInExpirationGracePeriod);
}

#[test]
fn challenge_offer_accept_offer_account_not_normal_status() {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
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
                "account": ACCOUNT,
                // Simulate the AccountCell is not in normal status.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked);
}

#[test]
fn challenge_offer_accept_offer_account_not_exists_in_inputs() {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate no AccountCell in inputs.

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_accept_offer_accept_multiple_offer_cells() {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
    // Simulate accepting multiple OfferCells at once.
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                "account": ACCOUNT,
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
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
                "account": ACCOUNT,
                // Simulate the AccountCell is not in normal status.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_accept_offer_account_capacity_mismatch() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            // Simulate the AccountCell.capacity decreased.
            "capacity": util::gen_account_cell_capacity(5) - 1,
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    challenge_tx(template.as_json(), Error::AccountCellChangeCapacityError);
}

#[test]
fn challenge_offer_accept_offer_account_data_mismatch() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            // Simulate the AccountCell.data is changed.
            "data": {
                "account": ACCOUNT,
                "expired_at": TIMESTAMP + 1000,
            },
            "witness": {
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent);
}

#[test]
fn challenge_offer_accept_offer_account_witness_mismatch() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            // Simulate the AccountCell.witness is changed.
            "witness": {
                "registered_at": TIMESTAMP - 1000,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified);
}

#[test]
fn challenge_offer_accept_offer_account_deleted() {
    let mut template = before_each();

    // outputs
    // Simulate the AccountCell is deleted.
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_accept_offer_account_create() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    // Simulate creating a new AccountCell
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": "yyyyy.bit",
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_accept_offer_account_mismatch() {
    let mut template = init_with_timestamp("accept_offer");

    // inputs
    push_input_offer_cell(
        &mut template,
        json!({
            "capacity": "200_100_000_000",
            "witness": {
                // Simulate the account in OfferCell is not match with in AccountCell.
                "account": "yyyyy.bit",
                "price": "200_000_000_000",
                "message": "Take my money.🍀"
            }
        }),
    );
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
                "account": ACCOUNT,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    // outputs
    push_common_outputs(&mut template);

    challenge_tx(template.as_json(), Error::OfferCellAccountMismatch);
}

#[test]
fn challenge_offer_accept_offer_no_income_cell() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_balance_cell(&mut template, 194_000_000_000 - 1, SELLER);

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure);
}

#[test]
fn challenge_offer_accept_offer_income_cell_lock_error() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_income_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate the IncomeCell.lock is not always_success lock.
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": "0x0000000000000000000000000000000000000000"
            },
            "witness": {
                "records": [
                    // It is a conversion in this transaction that the first record always belong to the creator of the IncomeCell.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "2_000_000_000"
                    }
                ]
            }
        }),
    );

    push_output_balance_cell(&mut template, 194_000_000_000 - 1, SELLER);

    challenge_tx(template.as_json(), Error::AlwaysSuccessLockIsRequired);
}

#[test]
fn challenge_offer_accept_offer_sellers_profit_wrong() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 194_000_000_000 - 1, SELLER);

    challenge_tx(template.as_json(), Error::ChangeError);
}

#[test]
fn challenge_offer_accept_offer_others_profit_wrong() {
    let mut template = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": BUYER,
                "manager_lock_args": BUYER
            },
            "data": {
                "account": ACCOUNT,
            },
            "witness": {
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    // It is a conversion in this transaction that the first record always belong to the creator of the IncomeCell.
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": "0x0000000000000000000000000000000000000000"
                        },
                        "capacity": "20_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(INVITER_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args(CHANNEL_LOCK_ARGS, None)
                        },
                        "capacity": "2_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        // Simulate some records of the IncomeCell is wrong.
                        "capacity": 2_000_000_000u64 - 1
                    }
                ]
            }
        }),
    );

    push_output_balance_cell(&mut template, 194_000_000_000, SELLER);

    challenge_tx(template.as_json(), Error::IncomeCellProfitMismatch);
}
