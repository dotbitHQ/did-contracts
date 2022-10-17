use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_parser::*;

#[test]
fn test_pre_register_refund() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "yyyyy.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    push_output_normal_cell(&mut template, 300_000_000_000, OWNER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

fn test_pre_register_refund_multi_target() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "yyyyy.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": MANAGER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    push_output_normal_cell(&mut template, 200_000_000_000, OWNER_WITHOUT_TYPE);
    push_output_normal_cell(&mut template, 100_000_000_000, MANAGER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

#[test]
fn challenge_pre_register_cell_in_outputs() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    // Simulate put PreAccountCells in outputs.
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );
    push_output_normal_cell(&mut template, 200_000_000_000, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_pre_register_refund_too_early() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                // Simulate refunding a PreAccountCell when it is not timeout.
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME + 1,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    push_output_normal_cell(&mut template, 100_000_000_000u64, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::PreRegisterIsNotTimeout)
}

#[test]
fn challenge_pre_register_refund_to_multiple_cells() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    // Simulate refunding the capacity of a single refund lock to multiple cells.
    push_output_normal_cell(&mut template, 10_000_000_000u64, OWNER_WITHOUT_TYPE);
    push_output_normal_cell(&mut template, 90_000_000_000u64, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure)
}

#[test]
fn challenge_pre_register_refund_capacity_not_enough() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_REFUND_WAITING_TIME,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
    );

    // outputs
    push_output_normal_cell(
        &mut template,
        // Simulate refunding incorrect capacity to the refund lock.
        100_000_000_000u64 - PRE_ACCOUNT_REFUND_AVAILABLE_FEE - 1,
        OWNER_WITHOUT_TYPE,
    );

    challenge_tx(template.as_json(), ErrorCode::PreRegisterRefundCapacityError)
}
