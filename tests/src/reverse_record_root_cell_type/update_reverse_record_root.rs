use das_types_std::constants::{DasLockType, DataType, Source};
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("update_reverse_record_root");

    template.push_contract_cell("ckb_sign.so", ContractType::SharedLib);
    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("tron_sign.so", ContractType::SharedLib);
    template.push_contract_cell("doge_sign.so", ContractType::SharedLib);

    template.push_config_cell(DataType::ConfigCellSMTNodeWhitelist, Source::CellDep);

    template
}

fn push_simple_input_reverse_record_root_cell(template: &mut TemplateGenerator) {
    template.restore_reverse_record(vec![
        json!({
            "address_payload": OWNER_2_WITHOUT_TYPE,
            "nonce": 5,
            "account": ACCOUNT_1,
        }),
        json!({
            "address_payload": OWNER_3_WITHOUT_TYPE,
            "nonce": 99,
            "account": ACCOUNT_1,
        }),
    ]);
    push_input_reverse_record_root_cell(template);
}

#[test]
fn test_reverse_record_root_update() {
    let mut template = before_each();

    // inputs
    push_simple_input_reverse_record_root_cell(&mut template);
    // The lock is in the white list.
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_1_WITHOUT_TYPE,
            // "prev_nonce": 0,
            // "prev_account": "",
            "next_account": ACCOUNT_1,
        }),
        false,
    );
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_2_WITHOUT_TYPE,
            "prev_nonce": 5,
            "prev_account": ACCOUNT_1,
            "next_account": ACCOUNT_2,
        }),
        false,
    );
    template.push_reverse_record(
        json!({
            "action": "remove",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_3_WITHOUT_TYPE,
            "prev_nonce": 99,
            "prev_account": ACCOUNT_1,
            "next_account": "",
        }),
        false,
    );
    push_output_reverse_record_root_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_reverse_record_root_update_change_capacity() {
    let mut template = before_each();

    // inputs
    push_simple_input_reverse_record_root_cell(&mut template);
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_1_WITHOUT_TYPE,
            "next_account": ACCOUNT_1,
        }),
        false,
    );
    let current_root = template.smt_with_history.current_root();
    template.push_output(
        json!({
            // Simulate changing the ReverseRecordRootCell.capacity in outputs.
            "capacity": REVERSE_RECORD_BASIC_CAPACITY - 1,
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), ErrorCode::CellCapacityMustBeConsistent);
}

#[test]
fn challenge_reverse_record_root_update_change_lock() {
    let mut template = before_each();

    // inputs
    push_simple_input_reverse_record_root_cell(&mut template);
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_1_WITHOUT_TYPE,
            "next_account": ACCOUNT_1,
        }),
        false,
    );
    let current_root = template.smt_with_history.current_root();
    template.push_output(
        json!({
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
            // Simulate changing the ReverseRecordRootCell.lock in outputs.
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": SUPER_LOCK_ARGS
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), ErrorCode::CellLockCanNotBeModified);
}

#[test]
fn challenge_reverse_record_root_update_store_mismatched_smt_root() {
    let mut template = before_each();

    // inputs
    push_simple_input_reverse_record_root_cell(&mut template);
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_1_WITHOUT_TYPE,
            "next_account": ACCOUNT_1,
        }),
        false,
    );
    // Simulate storing a mismatched SMT root in the ReverseRecordRootCell.data in outputs.
    let current_root = [1u8; 32];
    template.push_output(
        json!({
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), ErrorCode::SMTNewRootMismatch);
}

#[test]
fn challenge_reverse_record_root_update_witness_prev_nonce_error() {
    let mut template = before_each();

    // inputs
    push_simple_input_reverse_record_root_cell(&mut template);
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(
        json!({
            "action": "update",
            "sign_type": DasLockType::CKBSingle as u8,
            "address_payload": OWNER_2_WITHOUT_TYPE,
            // Simulate providing a invalid prev_nonce.
            "prev_nonce": 6,
            "prev_account": ACCOUNT_1,
            "next_account": ACCOUNT_2,
        }),
        true,
    );
    let current_root = template.smt_with_history.current_root();
    template.push_output(
        json!({
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{reverse-record-root-cell-type}}"
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), ErrorCode::SMTProofVerifyFailed);
}
