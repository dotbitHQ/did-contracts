use das_types::constants::*;
use das_types::prelude::*;
use serde_json::json;
use sparse_merkle_tree::H256;

use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::smt::SMTWithHistory;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

pub const TIMESTAMP: u64 = 1611200090u64;

fn init(name: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new("unit_test", Some(name.as_bytes().to_vec()));

    template.push_contract_cell("always_success", ContractType::DeployedContract);
    template.push_contract_cell("fake-secp256k1-blake160-signhash-all", ContractType::DeployedContract);
    template.push_contract_cell("test-env", ContractType::Contract);
    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

#[test]
fn test_parse_sub_account_witness_empty() {
    let mut template = init("test_parse_sub_account_witness_empty");

    push_input_test_env_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::WitnessEmpty);
}

#[test]
fn test_parse_sub_account_witness_create_only() {
    let mut template = init("test_parse_sub_account_witness_create_only");

    push_input_test_env_cell(&mut template);

    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, OWNER_1_WITHOUT_TYPE],
                [SUB_ACCOUNT_2, OWNER_2_WITHOUT_TYPE],
                [SUB_ACCOUNT_3, OWNER_3_WITHOUT_TYPE],
                [SUB_ACCOUNT_4, OWNER_4_WITHOUT_TYPE],
            ]
        }),
    );

    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_1)
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_2)
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_3)
    }));

    test_tx(template.as_json());
}

#[test]
fn test_parse_sub_account_witness_edit_only() {
    let mut template = init("test_parse_sub_account_witness_edit_only");
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": 0,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
    ]);

    push_input_test_env_cell(&mut template);

    template.push_sub_account_witness_v2(json!({
        "sign_expired_at": u64::MAX,
        "action": SubAccountAction::Edit.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        },
        // Simulate modifying owner.
        "edit_key": "owner",
        "edit_value": gen_das_lock_args(OWNER_2, Some(MANAGER_1))
    }));
    template.push_sub_account_witness_v2(json!({
        "sign_expired_at": u64::MAX,
        "action": SubAccountAction::Edit.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        },
        // Simulate modifying records.
        "edit_key": "records",
        "edit_value": [
            {
                "type": "address",
                "key": "eth",
                "label": "Personal",
                "value": "0x0000000000000000000000000000000000000000",
            },
        ]
    }));

    test_tx(template.as_json());
}

#[test]
fn test_parse_sub_account_witness_mixed() {
    let mut template = init("test_parse_sub_account_witness_mixed");
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": 0,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        }),
    ]);

    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            "account_list_smt_root": [
                // SUB_ACCOUNT_1 and SUB_ACCOUNT_3 is intentionally redundant.
                [SUB_ACCOUNT_1, OWNER_1_WITHOUT_TYPE],
                [SUB_ACCOUNT_2, OWNER_2_WITHOUT_TYPE],
                [SUB_ACCOUNT_3, OWNER_3_WITHOUT_TYPE],
                [SUB_ACCOUNT_4, OWNER_4_WITHOUT_TYPE],
            ]
        }),
    );

    push_input_test_env_cell(&mut template);

    template.push_sub_account_witness_v2(json!({
        "sign_expired_at": u64::MAX,
        "action": SubAccountAction::Edit.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        },
        // Simulate modifying owner.
        "edit_key": "expired_at",
        "edit_value": u64::MAX
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_2)
    }));
    template.push_sub_account_witness_v2(json!({
        "sign_expired_at": u64::MAX,
        "action": SubAccountAction::Edit.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": u64::MAX,
        },
        // Simulate modifying records.
        "edit_key": "records",
        "edit_value": [
            {
                "type": "address",
                "key": "eth",
                "label": "Personal",
                "value": "0x0000000000000000000000000000000000000000",
            },
        ]
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_4,
                "manager_lock_args": MANAGER_4
            },
            "account": SUB_ACCOUNT_4,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_4)
    }));

    test_tx(template.as_json());
}

fn get_compiled_proof(smt: &SMTWithHistory, account: &str) -> String {
    let key = H256::from(util::gen_smt_key_from_account(account));
    let proof = smt.get_compiled_proof(vec![key]);

    format!("0x{}", hex::encode(proof))
}

#[test]
fn test_parse_sub_account_rules_witness_empty() {
    let mut template = init("test_parse_sub_account_rules_witness_empty");

    push_input_test_env_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::WitnessEmpty);
}

#[test]
fn test_parse_sub_account_rules_witness_simple() {
    let mut template = init("test_parse_sub_account_rules_witness_simple");

    push_input_test_env_cell(&mut template);

    template.push_sub_account_rules_witness(
        DataType::SubAccountPriceRule,
        1,
        json!(
            [
                {
                    "index": 0,
                    "name": "Price of 1 Charactor Emoji DID",
                    "note": "",
                    "price": 100_000_000,
                    "status": 0,
                    "ast": {
                        "type": "operator",
                        "symbol": "and",
                        "expressions": [
                            {
                                "type": "operator",
                                "symbol": "==",
                                "expressions": [
                                    {
                                        "type": "variable",
                                        "name": "account_length",
                                    },
                                    {
                                        "type": "value",
                                        "value_type": "uint8",
                                        "value": 1,
                                    },
                                ],
                            },
                            {
                                "type": "function",
                                "name": "only_include_charset",
                                "arguments": [
                                    {
                                        "type": "variable",
                                        "name": "account_chars",
                                    },
                                    {
                                        "type": "value",
                                        "value_type": "charset_type",
                                        "value": "Emoji",
                                    }
                                ],
                            }
                        ]
                    }
                }
            ]
        ),
    );

    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, OWNER_1_WITHOUT_TYPE],
                [SUB_ACCOUNT_2, OWNER_2_WITHOUT_TYPE],
                [SUB_ACCOUNT_3, OWNER_3_WITHOUT_TYPE],
                [SUB_ACCOUNT_4, OWNER_4_WITHOUT_TYPE],
            ]
        }),
    );
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_value": get_compiled_proof(&smt, SUB_ACCOUNT_1)
    }));

    test_tx(template.as_json());
}
