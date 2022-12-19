use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::since_util::SinceFlag;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
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
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "yyyyy.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );

    // outputs
    push_output_normal_cell(&mut template, 300_000_000_000, OWNER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

#[test]
fn test_pre_register_refund_with_refund_lock() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_H,
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "yyyyy.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_H,
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_H,
    );
    push_input_normal_cell(&mut template, INPUT_CAPACITY_OF_REFUND_LOCK, OWNER_WITHOUT_TYPE);

    // outputs
    push_output_normal_cell(&mut template, INPUT_CAPACITY_OF_REFUND_LOCK, OWNER_WITHOUT_TYPE);
    push_output_normal_cell(&mut template, 300_000_000_000, OWNER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

#[test]
fn challenge_pre_register_refund_multi_target() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_1_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_2_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );

    // outputs
    push_output_normal_cell(&mut template, 100_000_000_000, OWNER_1_WITHOUT_TYPE);
    push_output_normal_cell(&mut template, 100_000_000_000, OWNER_2_WITHOUT_TYPE);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::RefundLockMustBeUnique)
}

#[test]
fn challenge_pre_register_refund_outputs_not_clean() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_TIMEOUT_LIMIT,
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );

    // outputs
    // Simulate create PreAccountCells in outputs.
    push_output_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
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
fn challenge_pre_register_refund_since_relative_flag_error() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        // Simulate refunding a PreAccountCell with wrong since.
        gen_since(SinceFlag::Absolute, SinceFlag::Timestamp, DAY_SEC),
    );

    // outputs
    push_output_normal_cell(&mut template, 100_000_000_000u64, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::SinceMismatch)
}

#[test]
fn challenge_pre_register_refund_since_metric_flag_error() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        // Simulate refunding a PreAccountCell with wrong since.
        gen_since(SinceFlag::Relative, SinceFlag::Height, DAY_SEC),
    );

    // outputs
    push_output_normal_cell(&mut template, 100_000_000_000u64, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::SinceMismatch)
}

#[test]
fn challenge_pre_register_refund_since_value_error() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        // Simulate refunding a PreAccountCell with wrong since.
        gen_since(SinceFlag::Relative, SinceFlag::Timestamp, DAY_SEC - 1),
    );

    // outputs
    push_output_normal_cell(&mut template, 100_000_000_000u64, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::SinceMismatch)
}

#[test]
fn challenge_pre_register_refund_with_refund_lock_since_value_error() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "zzzzz.bit",
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        // Simulate refunding a PreAccountCell with wrong since.
        gen_since(SinceFlag::Relative, SinceFlag::Timestamp, HOUR_SEC - 1),
    );
    push_input_normal_cell(&mut template, INPUT_CAPACITY_OF_REFUND_LOCK, OWNER_WITHOUT_TYPE);

    // outputs
    push_output_normal_cell(&mut template, INPUT_CAPACITY_OF_REFUND_LOCK, OWNER_WITHOUT_TYPE);
    push_output_normal_cell(&mut template, 100_000_000_000, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), PreAccountCellErrorCode::SinceMismatch)
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
                "refund_lock": {
                    "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                    "args": OWNER_WITHOUT_TYPE
                },
            }
        }),
        *SINCE_1_D,
    );

    // outputs
    push_output_normal_cell(
        &mut template,
        // Simulate refunding incorrect capacity to the refund lock.
        100_000_000_000u64 - PRE_ACCOUNT_REFUND_AVAILABLE_FEE - 1,
        OWNER_WITHOUT_TYPE,
    );

    challenge_tx(template.as_json(), PreAccountCellErrorCode::RefundCapacityError)
}

#[test]
fn challenge_pre_register_refund_to_das_lock_without_type() {
    let mut template = init_for_refund();

    // inputs
    push_input_pre_account_cell(
        &mut template,
        json!({
            "capacity": 100_000_000_000u64,
            "witness": {
                "account": "xxxxx.bit",
                "created_at": TIMESTAMP - PRE_ACCOUNT_TIMEOUT_LIMIT,
                "refund_lock": {
                    "code_hash": "{{fake-das-lock}}",
                    "args": gen_das_lock_args(SELLER, None)
                },
            }
        }),
        *SINCE_1_D,
    );

    // outputs
    push_output_balance_cell_without_type(&mut template, 100_000_000_000u64, SELLER);

    challenge_tx(template.as_json(), ErrorCode::BalanceCellFoundSomeOutputsLackOfType)
}
