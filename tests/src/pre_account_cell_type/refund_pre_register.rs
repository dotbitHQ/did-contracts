use super::common::*;
use crate::util::{self, accounts::*, constants::*, error::Error, template_common_cell::*, template_parser::*};
use das_types_std::constants::*;
use serde_json::json;

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
                "created_at": TIMESTAMP - 86400,
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
                "created_at": TIMESTAMP - 86400,
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
                "created_at": TIMESTAMP - 86400,
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
