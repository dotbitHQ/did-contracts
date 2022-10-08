use das_types_std::constants::*;
use das_types_std::prelude::*;
use serde_json::json;

use crate::util::accounts::*;
use crate::util::constants::*;
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

    template.push_config_cell(DataType::ConfigCellMain, Source::CellDep);

    template
}

#[test]
fn parse_sub_account_witness_empty() {
    let mut template = init("test_parse_sub_account_witness_empty");

    push_input_test_env_cell(&mut template);

    challenge_tx(template.as_json(), ErrorCode::WitnessEmpty);
}

#[test]
fn parse_sub_account_witness_create() {
    let mut template = init("test_parse_sub_account_witness_create");

    push_input_test_env_cell(&mut template);

    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_3,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn parse_sub_account_witness_edit() {
    let mut template = init("test_parse_sub_account_witness_edit");
    template.restore_sub_account(vec![
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

    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
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
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
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
        }),
    );
    template.push_sub_account_witness(
        SubAccountActionType::Edit,
        json!({
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
        }),
    );

    test_tx(template.as_json());
}
