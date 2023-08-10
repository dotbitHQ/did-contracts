use das_types_std::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

const INIT_DAS_PROFIT: u64 = 10_000_000_000;
const INIT_OWNER_PROFIT: u64 = 10_000_000_000;

fn before_each_without_custom_script() -> TemplateGenerator {
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
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell(
        &mut template,
        INIT_DAS_PROFIT,
        INIT_OWNER_PROFIT,
        SubAccountConfigFlag::Manual,
    );

    template
}

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    template.push_contract_cell("test-custom-script", ContractType::Contract);
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
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
        json!({
            "lock": {
                "owner_lock_args": OWNER_3,
                "manager_lock_args": MANAGER_3
            },
            "account": SUB_ACCOUNT_3,
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        }),
    ]);
    push_simple_input_sub_account_cell_with_custom_script(
        &mut template,
        INIT_DAS_PROFIT,
        INIT_OWNER_PROFIT,
        SCRIPT_ARGS,
    );
    push_input_normal_cell(&mut template, 100_000_000_000, OWNER);

    template
}

#[test]
fn test_sub_account_update_without_custom_script() {
    let mut template = before_each_without_custom_script();

    // outputs
    let smt = push_commen_mint_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Edit.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
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
        "edit_key": "manager",
        // Simulate modifying manager.
        "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
    }));
    template.push_sub_account_witness_v3(json!({
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
    push_simple_output_sub_account_cell(
        &mut template,
        INIT_DAS_PROFIT + SUB_ACCOUNT_NEW_PRICE,
        INIT_OWNER_PROFIT,
        SubAccountConfigFlag::Manual,
    );

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_update_with_custom_script() {
    let mut template = before_each();

    // outputs
    let smt = push_commen_mint_sign_witness(&mut template);
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Edit.to_string(),
        "sign_role": "0x00",
        "sign_expired_at": TIMESTAMP,
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
        "edit_key": "manager",
        // Simulate modifying manager.
        "edit_value": gen_das_lock_args(OWNER_1, Some(MANAGER_2))
    }));
    template.push_sub_account_witness_v3(json!({
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

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(
        &mut template,
        INIT_DAS_PROFIT + das_profit,
        INIT_OWNER_PROFIT + owner_profit,
        SCRIPT_ARGS,
    );
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_update_parent_not_in_normal_status() {
    let mut template = init_update();

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                // Simulate using the AccountCell that is not in normal status.
                "status": (AccountStatus::Selling as u8),
                "enable_sub_account": 1,
            }
        }),
    );

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    let smt = push_commen_mint_sign_witness(&mut template);
    template.push_sub_account_witness_v3(json!({
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
    push_common_output_cells(&mut template, 1, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), AccountCellErrorCode::AccountCellStatusLocked);
}

#[test]
fn challenge_sub_account_update_parent_expired() {
    let mut template = init_update();

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1000,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "account": ACCOUNT_1,
                // Simulate using the AccountCell that is expired.
                "expired_at": TIMESTAMP - 1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    let smt = template.push_sub_account_mint_sign_witness(
        DataType::SubAccountMintSign,
        json!({
            "version": 1,
            "expired_at": TIMESTAMP - 1,
            "account_list_smt_root": [
                [SUB_ACCOUNT_1, gen_das_lock_args(OWNER_1, Some(MANAGER_1))],
                [SUB_ACCOUNT_2, gen_das_lock_args(OWNER_2, Some(MANAGER_2))],
                [SUB_ACCOUNT_3, gen_das_lock_args(OWNER_3, Some(MANAGER_3))],
                [SUB_ACCOUNT_4, gen_das_lock_args(OWNER_4, Some(MANAGER_4))],
            ]
        }),
    );
    template.push_sub_account_witness_v3(json!({
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
    push_common_output_cells(&mut template, 1, SubAccountConfigFlag::Manual);

    challenge_tx(
        template.as_json(),
        AccountCellErrorCode::AccountCellInExpirationGracePeriod,
    );
}

#[test]
fn challenge_sub_account_update_parent_not_enable_feature() {
    let mut template = init_update();

    // cell_deps
    push_dep_account_cell(
        &mut template,
        json!({
            "header": {
                "height": HEIGHT - 1000,
                "timestamp": TIMESTAMP - DAY_SEC,
            },
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                // Simulate the parent AccountCell has not enable sub-account feature.
                "enable_sub_account": 0,
            }
        }),
    );

    // inputs
    push_simple_input_sub_account_cell(&mut template, 0, 0, SubAccountConfigFlag::Manual);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    let smt = push_commen_mint_sign_witness(&mut template);
    template.push_sub_account_witness_v3(json!({
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
    push_common_output_cells(&mut template, 1, SubAccountConfigFlag::Manual);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountFeatureNotEnabled);
}
