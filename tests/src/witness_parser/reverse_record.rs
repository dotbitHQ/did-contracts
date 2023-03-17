use das_types_std::constants::*;
use das_types_std::prelude::*;
use serde_json::json;

use crate::util::accounts::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

pub const TIMESTAMP: u64 = 1611200090u64;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);

    template
}

#[test]
fn parse_reverse_record_witness_empty() {
    let mut template = init("test_parse_reverse_record_witness_empty");

    push_input_test_env_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::WitnessEmpty);
}

#[test]
fn parse_reverse_record_witness_update_only() {
    let mut template = init("test_parse_reverse_record_witness_update_only");

    push_input_test_env_cell(&mut template);

    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_1_WITHOUT_TYPE,
        "prev_nonce": 0,
        "prev_account": "",
        "next_account": ACCOUNT_1,
    }), false);
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_2_WITHOUT_TYPE,
        "prev_nonce": 5,
        "prev_account": ACCOUNT_1,
        "next_account": ACCOUNT_2,
    }), false);

    test_tx(template.as_json());
}

#[test]
fn parse_reverse_record_witness_remove_only() {
    let mut template = init("test_parse_reverse_record_witness_remove_only");

    template.restore_reverse_record(vec![
        json!({
            "address_payload": OWNER_2_WITHOUT_TYPE,
            "nonce": 5,
            "account": ACCOUNT_2,
        }),
        json!({
            "address_payload": OWNER_3_WITHOUT_TYPE,
            "nonce": 99,
            "account": ACCOUNT_3,
        }),
    ]);

    push_input_test_env_cell(&mut template);

    template.push_reverse_record(json!({
        "action": "remove",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_2_WITHOUT_TYPE,
        "prev_nonce": 5,
        "prev_account": ACCOUNT_2,
        "next_account": "",
    }), false);
    template.push_reverse_record(json!({
        "action": "remove",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_3_WITHOUT_TYPE,
        "prev_nonce": 99,
        "prev_account": ACCOUNT_3,
        "next_account": "",
    }), false);

    test_tx(template.as_json());
}

#[test]
fn parse_reverse_record_witness_mixed() {
    let mut template = init("test_parse_reverse_record_witness_mixed");

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

    push_input_test_env_cell(&mut template);

    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_1_WITHOUT_TYPE,
        "prev_nonce": 0,
        "prev_account": "",
        "next_account": ACCOUNT_1,
    }), false);
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_2_WITHOUT_TYPE,
        "prev_nonce": 5,
        "prev_account": ACCOUNT_1,
        "next_account": ACCOUNT_2,
    }), false);
    template.push_reverse_record(json!({
        "action": "remove",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_3_WITHOUT_TYPE,
        "prev_nonce": 99,
        "prev_account": ACCOUNT_1,
        "next_account": "",
    }), false);

    test_tx(template.as_json());
}
