use das_types_std::constants::{DasLockType, DataType, Source};
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init("update_reverse_record_root");

    template.push_contract_cell("eth_sign.so", ContractType::SharedLib);
    template.push_contract_cell("ckb_sign.so", ContractType::SharedLib);
    template.push_contract_cell("tron_sign.so", ContractType::SharedLib);

    template.push_config_cell(DataType::ConfigCellSMTNodeWhitelist, Source::CellDep);

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
    // The lock is in the white list.
    push_input_normal_cell(&mut template, 0, OWNER_1_WITHOUT_TYPE);

    // outputs
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_1_WITHOUT_TYPE,
        // "prev_nonce": 0,
        // "prev_account": "",
        "next_account": ACCOUNT_1,
    }));
    template.push_reverse_record(json!({
        "action": "update",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_2_WITHOUT_TYPE,
        "prev_nonce": 5,
        "prev_account": ACCOUNT_1,
        "next_account": ACCOUNT_2,
    }));
    template.push_reverse_record(json!({
        "action": "remove",
        "sign_type": DasLockType::CKBSingle as u8,
        "address_payload": OWNER_3_WITHOUT_TYPE,
        "prev_nonce": 99,
        "prev_account": ACCOUNT_1,
        "next_account": "",
    }));
    push_output_reverse_record_root_cell(&mut template);

    test_tx(template.as_json());
}
