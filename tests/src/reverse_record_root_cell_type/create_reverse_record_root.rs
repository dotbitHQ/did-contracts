use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let template = init("create_reverse_record_root");

    template
}

#[test]
fn test_reverse_record_root_create() {
    let mut template = before_each();

    // inputs
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    push_output_reverse_record_root_cell(&mut template);

    test_tx(template.as_json());
}

#[test]
fn challenge_reverse_record_root_create_without_super_lock() {
    let mut template = before_each();

    // inputs
    // Simulate create a new root cell without super lock.
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    push_output_reverse_record_root_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::SuperLockIsRequired);
}

#[test]
fn challenge_reverse_record_root_without_always_success_lock() {
    let mut template = before_each();

    // inputs
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    let current_root = template.smt_with_history.current_root();
    template.push_output(
        json!({
            "capacity": REVERSE_RECORD_BASIC_CAPACITY,
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

    challenge_tx(template.as_json(), ErrorCode::AlwaysSuccessLockIsRequired);
}

#[test]
fn challenge_reverse_record_root_without_empty_root() {
    let mut template = before_each();

    // inputs
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    // Simulate create a new root cell with not empty root.
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

    challenge_tx(
        template.as_json(),
        ReverseRecordRootCellErrorCode::InitialOutputsDataError,
    );
}
