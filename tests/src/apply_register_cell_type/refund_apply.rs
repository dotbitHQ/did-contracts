use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
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
                "args": OWNER_WITHOUT_TYPE
            },
            "data": {
                "height": HEIGHT - APPLY_MAX_WAITING_BLOCK - 1,
                "timestamp": TIMESTAMP,
            }
        }),
    );

    push_output_normal_cell(&mut template, 19_900_000_000, OWNER_WITHOUT_TYPE);

    test_tx(template.as_json())
}

#[test]
fn challenge_apply_register_refund_too_early() {
    let mut template = before();

    push_input_apply_register_cell(
        &mut template,
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                "args": OWNER_WITHOUT_TYPE
            },
            "data": {
                // Simulate refunding the ApplyRegisterCell too early ...
                "height": HEIGHT - APPLY_MAX_WAITING_BLOCK,
                "timestamp": TIMESTAMP,
            }
        }),
    );

    push_output_normal_cell(&mut template, 19_900_000_000, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterRefundNeedWaitLonger)
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
            "data": {
                "height": HEIGHT - APPLY_MAX_WAITING_BLOCK - 1,
                "timestamp": TIMESTAMP,
            }
        }),
    );

    // Simulate refunding the ApplyRegisterCell with wrong capacity ...
    push_output_normal_cell(&mut template, 19_900_000_000 - 1, OWNER_WITHOUT_TYPE);

    challenge_tx(template.as_json(), ErrorCode::ApplyRegisterRefundCapacityError)
}
