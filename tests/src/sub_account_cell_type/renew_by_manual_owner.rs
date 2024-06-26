use das_types::constants::*;
use serde_json::json;

use super::common::*;
use crate::util;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    // cell_deps
    push_simple_dep_account_cell(&mut template);

    // inputs
    template.restore_sub_account_v1(vec![
        json!({
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        }),
    ]);
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    push_input_normal_cell(&mut template, TOTAL_PAID, OWNER);

    template
}

#[test]
fn test_sub_account_renew_flag_manual_by_owner_without_dpoint() {
    let mut template = before_each();

    // outputs
    let smt = push_commen_renew_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_1),
        }
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC * 2,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_2),
        }
    }));
    push_common_output_cells(&mut template, 3, SubAccountConfigFlag::Manual);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_renew_flag_manual_by_owner_with_dpoint() {
    let mut template = before_each();

    template.push_contract_cell("dpoint-cell-type", ContractType::Contract);
    template.push_config_cell(DataType::ConfigCellDPoint, Source::CellDep);

    // Payment in DPoint
    push_input_dpoint_cell(&mut template, TOTAL_PAID_DP, OWNER);

    // outputs
    let smt = push_commen_renew_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_1),
        }
    }));
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_2,
                "manager_lock_args": MANAGER_2
            },
            "account": SUB_ACCOUNT_2,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC * 2,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_2),
        }
    }));
    push_common_output_cells(&mut template, 3, SubAccountConfigFlag::Manual);

    // Profit in DPoint
    let das_profit = SUB_ACCOUNT_NEW_PRICE * 3;
    push_output_dpoint_cell(&mut template, TOTAL_PAID_DP - das_profit, OWNER);
    push_output_dpoint_cell(&mut template, das_profit, DP_TRANSFER_WHITELIST_1);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_renew_flag_manual_multi_sign_role() {
    let mut template = before_each();

    // outputs
    let renew_smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountRenewSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            // Simulate the SubAccountMintSign and the SubAccountRenewSign have different sign_role.
            "sign_role": "0x01",
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
            ]
        }),
    );
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
            "rest": get_compiled_proof(&renew_smt, SUB_ACCOUNT_1),
        }
    }));
    let sign_smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP + DAY_SEC,
            // Simulate the SubAccountMintSign and the SubAccountRenewSign have different sign_role.
            "sign_role": "0x00",
            "account_list_smt_root": [
                [SUB_ACCOUNT_4, gen_das_lock_args(OWNER_4, Some(MANAGER_4))],
            ]
        }),
    );
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
        "edit_key": "manual",
        "edit_value": get_compiled_proof(&sign_smt, SUB_ACCOUNT_4)
    }));
    push_common_output_cells(&mut template, 2, SubAccountConfigFlag::Manual);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::MultipleSignRolesIsNotAllowed,
    );
}

#[test]
fn challenge_sub_account_renew_flag_manual_expired_at_less_than_one_year() {
    let mut template = before_each();

    // outputs
    let smt = push_commen_renew_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            // Simulate the expired_at is less than one year.
            "expired_at": TIMESTAMP + YEAR_SEC - 1,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_1),
        }
    }));
    push_common_output_cells(&mut template, 1, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::ExpirationYearsTooShort);
}

#[test]
fn challenge_sub_account_renew_flag_manual_no_profit_record() {
    let mut template = before_each();

    // outputs
    let smt = push_commen_renew_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_1),
        }
    }));

    let das_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_RENEW_PRICE, 1);
    // Simulate forget record correct profit in the outputs_data of the SubAccountCell
    push_simple_output_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    push_output_normal_cell(&mut template, TOTAL_PAID - das_profit, OWNER);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountProfitError);
}

#[test]
fn challenge_sub_account_renew_flag_manual_profit_not_match_capacity() {
    let mut template = before_each();

    // outputs
    let smt = push_commen_mint_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Renew.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": SUB_ACCOUNT_1,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP,
        },
        "edit_key": "manual",
        "edit_value": {
            "expired_at": TIMESTAMP + YEAR_SEC,
            "rest": get_compiled_proof(&smt, SUB_ACCOUNT_1),
        }
    }));

    let das_profit = util::gen_sub_account_register_fee(SUB_ACCOUNT_RENEW_PRICE, 1);
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            // Simulate forget put profit into the capacity of the SubAccountCell
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + das_profit - 1,
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit
            }
        }),
    );

    push_output_normal_cell(&mut template, TOTAL_PAID - das_profit, OWNER);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountCellCapacityError);
}
