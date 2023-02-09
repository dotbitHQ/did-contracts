use das_types_std::constants::DasLockType;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let template = init("create_reverse_record_root");

    template
}

#[test]
fn test_reverse_record_root_update() {
    let mut template = before_each();

    // inputs
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
    push_input_reverse_record_root_cell(&mut template);
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "sign_expired_at": TIMESTAMP + MONTH_SEC,
        "address_payload": OWNER_1_WITHOUT_TYPE,
        "prev_nonce": 0,
        "prev_account": "",
        "next_account": ACCOUNT_1,
    }));
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "sign_expired_at": TIMESTAMP + MONTH_SEC,
        "address_payload": OWNER_2_WITHOUT_TYPE,
        "prev_nonce": 5,
        "prev_account": ACCOUNT_1,
        "next_account": ACCOUNT_2,
    }));
    template.push_reverse_record(json!({
        "action": "remove",
        "sign_type": DasLockType::CKBSingle as u8,
        "sign_expired_at": TIMESTAMP + MONTH_SEC,
        "address_payload": OWNER_3_WITHOUT_TYPE,
        "prev_nonce": 99,
        "prev_account": ACCOUNT_1,
        "next_account": "",
    }));
    push_output_reverse_record_root_cell(&mut template);

    test_tx(template.as_json());
}
