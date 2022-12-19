use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::since_util::SinceFlag;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before() -> TemplateGenerator {
    init("refund_apply")
}

#[test]
fn test_apply_register_refund() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_1_WITHOUT_TYPE
            },
        }),
        *SINCE_MAX_HEIGHT,
    );

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_1_WITHOUT_TYPE
            },
        }),
        *SINCE_MAX_HEIGHT,
    );

    push_output_normal_cell(&mut template, 40_000_000_000 - APPLY_REFUND_REWARD, OWNER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

#[test]
fn challenge_apply_register_refund_since_relative_flag_error() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_WITHOUT_TYPE
            },
        }),
        // Simulate refunding a ApplyRegisterCell with wrong since.
        gen_since(SinceFlag::Absolute, SinceFlag::Height, APPLY_MAX_WAITING_BLOCK),
    );

    push_output_normal_cell(&mut template, 20_000_000_000 - APPLY_REFUND_REWARD, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterSinceMismatch)
}

#[test]
fn challenge_apply_register_refund_since_metric_flag_error() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_WITHOUT_TYPE
            },
        }),
        // Simulate refunding a ApplyRegisterCell with wrong since.
        gen_since(SinceFlag::Relative, SinceFlag::Timestamp, APPLY_MAX_WAITING_BLOCK),
    );

    push_output_normal_cell(&mut template, 20_000_000_000 - APPLY_REFUND_REWARD, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterSinceMismatch)
}

#[test]
fn challenge_apply_register_refund_since_value_error() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_WITHOUT_TYPE
            },
        }),
        // Simulate refunding a ApplyRegisterCell with wrong since.
        gen_since(SinceFlag::Relative, SinceFlag::Height, APPLY_MAX_WAITING_BLOCK - 1),
    );

    push_output_normal_cell(&mut template, 20_000_000_000 - APPLY_REFUND_REWARD, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterSinceMismatch)
}

#[test]
fn challenge_apply_register_refund_capacity_error() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_WITHOUT_TYPE
            },
        }),
        *SINCE_MAX_HEIGHT,
    );

    // Simulate refunding the ApplyRegisterCell with wrong capacity ...
    push_output_normal_cell(
        &mut template,
        20_000_000_000 - APPLY_REFUND_REWARD - 1,
        OWNER_WITHOUT_TYPE,
    );

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterRefundCapacityError)
}
