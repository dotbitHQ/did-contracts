use das_types::constants::*;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::error::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init_update();

    template.push_contract_cell("test-custom-script", ContractType::Contract);
    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_input_sub_account_cell_with_custom_script(&mut template, 0, 0, SCRIPT_ARGS);
    push_input_normal_cell(&mut template, 100_000_000_000, OWNER);

    template
}

#[test]
fn test_sub_account_create_flag_custom_script_with_args() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
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
        "edit_key": "custom_script",
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
        "edit_key": "custom_script",
    }));
    push_common_output_cells_with_custom_script(&mut template, 3);

    test_tx(template.as_json())
}

#[test]
fn test_sub_account_create_flag_custom_script_without_args() {
    let mut template = init_update();

    template.push_contract_cell("test-custom-script", ContractType::Contract);
    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_input_sub_account_cell_with_custom_script(&mut template, 0, 0, "");
    push_input_normal_cell(&mut template, 100_000_000_000, OWNER);

    // outputs
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
        "edit_key": "custom_script",
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
        "edit_key": "custom_script",
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
        "edit_key": "custom_script",
    }));
    let total_profit = calculate_sub_account_custom_price(3);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, "");
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_create_flag_custom_script_flag_not_consistent() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                // Simulate modifying the flag of the SubAccountCell.
                "flag": SubAccountConfigFlag::Manual as u8,
            }
        }),
    );
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_script_code_hash_not_consistent() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                // Simulate modifying the custom script of the SubAccountCell.
                "custom_script": "0x01000000742d637573746f6d2d736372000000",
                "script_args": SCRIPT_ARGS
            }
        }),
    );
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_script_args_not_consistent() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, "");
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountCellConsistencyError,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_mix_custom_rule() {
    let mut template = before_each();

    // outputs
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
        // Simulate mix custom rule mint into custom script mint.
        "edit_key": "custom_rule",
        "edit_value": "0x00000000000000000000000000000000000000000000000000000000"
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, SCRIPT_ARGS);
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::WitnessEditKeyInvalid);
}

#[test]
fn challenge_sub_account_create_flag_custom_script_different_lock_for_normal_cells() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, SCRIPT_ARGS);
    // Simulate change to a different lock which is not the same as the lock in inputs.
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER_1);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SubAccountNormalCellLockLimit,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_das_profit_not_enough() {
    let mut template = before_each();

    // outputs
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
        "edit_key": "custom_script",
    }));

    let total_profit = calculate_sub_account_custom_price(1);
    // Simulate the profit of DAS is less than expected value.
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE - 1;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, SCRIPT_ARGS);
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(template.as_json(), SubAccountCellErrorCode::SubAccountProfitError);
}

#[test]
fn challenge_sub_account_create_flag_custom_script_spend_balance_cell_1() {
    let mut template = init_update();

    template.push_contract_cell("test-custom-script", ContractType::Contract);
    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_input_sub_account_cell_with_custom_script(&mut template, 0, 0, "");
    // Simulate spending the BalanceCells of the parent AccountCell owner.
    push_input_balance_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
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
        "edit_key": "custom_script",
    }));
    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, "");
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_spend_balance_cell_2() {
    let mut template = init_update();

    template.push_contract_cell("test-custom-script", ContractType::Contract);
    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_input_sub_account_cell_with_custom_script(&mut template, 0, 0, "");
    // Simulate spending the BalanceCells of the parent AccountCell owner.
    push_input_balance_cell(&mut template, 10_000_000_000, OWNER_4);

    // outputs
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
        "edit_key": "custom_script",
    }));
    let total_profit = calculate_sub_account_custom_price(1);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit, "");
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(
        template.as_json(),
        SubAccountCellErrorCode::SomeCellWithDasLockMayBeAbused,
    );
}

#[test]
fn challenge_sub_account_create_flag_custom_script_create_empty() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness_v2(json!({
        "action": SubAccountAction::Create.to_string(),
        "sub_account": {
            "lock": {
                "owner_lock_args": OWNER_1,
                "manager_lock_args": MANAGER_1
            },
            "account": ".xxxxx.bit",
            "suffix": SUB_ACCOUNT_SUFFIX,
            "registered_at": TIMESTAMP,
            "expired_at": TIMESTAMP + YEAR_SEC,
        },
        "edit_key": "custom_script",
    }));
    push_common_output_cells_with_custom_script(&mut template, 3);

    challenge_tx(template.as_json(), ErrorCode::AccountIsTooShort)
}
