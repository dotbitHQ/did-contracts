use ::did_cell_type::error::ErrorCode;
use das_types::constants::Source;
use serde_json::json;

use crate::account_cell_type::common;
use crate::did_cell_type::tmpl;
use crate::util::accounts::{ACCOUNT_1, OWNER, SENDER};
use crate::util::constants::{DAS_WALLET_LOCK_ARGS, TIMESTAMP};
use crate::util::template_common_cell::{
    push_input_account_cell, push_input_balance_cell, push_output_account_cell, push_output_balance_cell,
    push_output_income_cell,
};
use crate::util::template_generator::ContractType;
use crate::util::template_parser::{challenge_tx, test_tx};

#[test]
fn test_renewal_account() {
    let duration: u64 = 365 * 24 * 60 * 60;
    let mut base = common::init_for_renew("renew_account", Some("0x00"));
    base.push_contract_cell("did-cell-type", ContractType::Contract);
    push_input_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_output_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP + duration,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_input_balance_cell(&mut base, 1_000_000_000_000, OWNER);
    push_output_balance_cell(&mut base, 500_000_000_000, OWNER);
    push_output_income_cell(
        &mut base,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 500_000_000_000u64,
                    }
                ]
            }
        }),
    );
    tmpl::push_did_cell(&mut base, ACCOUNT_1.as_bytes(), TIMESTAMP, 2, Source::Input);
    tmpl::push_did_cell(&mut base, ACCOUNT_1.as_bytes(), TIMESTAMP + duration, 2, Source::Output);

    test_tx(base.as_json());
}

#[test]
fn challenge_renewal_account() {
    let duration: u64 = 365 * 24 * 60 * 60;
    let mut base = common::init_for_renew("renew_account", Some("0x00"));
    base.push_contract_cell("did-cell-type", ContractType::Contract);
    push_input_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_output_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP + duration,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_input_balance_cell(&mut base, 1_000_000_000_000, OWNER);
    push_output_balance_cell(&mut base, 500_000_000_000, OWNER);
    push_output_income_cell(
        &mut base,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 500_000_000_000u64,
                    }
                ]
            }
        }),
    );
    tmpl::push_did_cell(&mut base, ACCOUNT_1.as_bytes(), TIMESTAMP, 2, Source::Input);
    tmpl::push_did_cell(
        &mut base,
        ACCOUNT_1.as_bytes(),
        TIMESTAMP + duration + 1,
        2,
        Source::Output,
    );

    challenge_tx(base.as_json(), ErrorCode::ExpireAtNotEqual);
}

#[test]
fn challenge_renewal_account_args() {
    let duration: u64 = 365 * 24 * 60 * 60;
    let mut base = common::init_for_renew("renew_account", Some("0x00"));
    base.push_contract_cell("did-cell-type", ContractType::Contract);
    push_input_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_output_account_cell(
        &mut base,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "expired_at": TIMESTAMP + duration,
            },
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "status": 0x99
            }
        }),
    );

    push_input_balance_cell(&mut base, 1_000_000_000_000, OWNER);
    push_output_balance_cell(&mut base, 500_000_000_000, OWNER);
    push_output_income_cell(
        &mut base,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": 500_000_000_000u64,
                    }
                ]
            }
        }),
    );
    tmpl::push_did_cell_with_args(&mut base, ACCOUNT_1.as_bytes(), TIMESTAMP, 2, Source::Input, [1u8; 32]);
    tmpl::push_did_cell_with_args(&mut base, ACCOUNT_1.as_bytes(), TIMESTAMP, 2, Source::Output, [2u8; 32]);

    challenge_tx(base.as_json(), ErrorCode::WrongTypeArgs);
}
