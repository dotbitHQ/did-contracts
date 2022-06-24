use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_simple_input_account_cell(&mut template);
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    template
}

fn push_simple_input_account_cell(template: &mut TemplateGenerator) {
    push_input_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_input_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit
            }
        }),
    );
}

fn push_simple_output_account_cell(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, das_profit: u64, owner_profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit
            }
        }),
    );
}

fn push_common_output_cells(template: &mut TemplateGenerator) {
    let das_profit = calculate_sub_account_cost(template);
    push_simple_output_account_cell(template);
    push_simple_output_sub_account_cell(template, das_profit, 0);
    push_output_normal_cell(template, 10_000_000_000 - das_profit, OWNER);
}

fn calculate_sub_account_cost(template: &TemplateGenerator) -> u64 {
    SUB_ACCOUNT_NEW_PRICE * template.sub_account_outer_witnesses.len() as u64
}

fn before_each_with_custom_script() -> TemplateGenerator {
    let mut template = init_create("create_sub_account", Some("0x00"));

    template.push_contract_cell("test-custom-script", false);
    push_simple_dep_account_cell(&mut template);

    // inputs
    push_simple_input_sub_account_cell_with_custom_script(&mut template, 0, 0);
    push_input_normal_cell(&mut template, 100_000_000_000, OWNER);

    template
}

fn push_simple_dep_account_cell(template: &mut TemplateGenerator) {
    push_dep_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_input_sub_account_cell_with_custom_script(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
) {
    let current_root = template.smt_with_history.current_root();
    push_input_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                // 01 means hash type is type.
                // 746573742d637573746f6d2d736372697074 means args of type ID 578232827c6e74b39bf8894694ae4e0884a44df295d5e15770e4c06869cee1d4 of custom script.
                "custom_script": "0x01746573742d637573746f6d2d736372697074"
            }
        }),
    );
}

fn push_simple_output_sub_account_cell_with_custom_script(
    template: &mut TemplateGenerator,
    das_profit: u64,
    owner_profit: u64,
) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                // 01 means hash type is type.
                // 746573742d637573746f6d2d736372697074 means args of type ID 578232827c6e74b39bf8894694ae4e0884a44df295d5e15770e4c06869cee1d4 of custom script.
                "custom_script": "0x01746573742d637573746f6d2d736372697074"
            }
        }),
    );
}

fn push_common_output_cells_with_custom_script(template: &mut TemplateGenerator) {
    let total_profit = calculate_sub_account_custom_price(template);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(template, das_profit, owner_profit);
    push_output_normal_cell(template, 100_000_000_000 - total_profit, OWNER);
}

fn calculate_sub_account_custom_price(template: &TemplateGenerator) -> u64 {
    SUB_ACCOUNT_NEW_CUSTOM_PRICE * template.sub_account_outer_witnesses.len() as u64
}

#[test]
fn test_sub_account_create_without_custom_script() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
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
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_3,
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
                "account": SUB_ACCOUNT_4,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_create_parent_not_in_normal_status() {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                // Simulate using the AccountCell that is not in normal status.
                "status": (AccountStatus::Selling as u8),
                "enable_sub_account": 1,
            }
        }),
    );
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let das_profit = calculate_sub_account_cost(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "status": (AccountStatus::Selling as u8),
                "enable_sub_account": 1,
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, das_profit, 0);
    push_output_normal_cell(&mut template, 10_000_000_000 - das_profit, OWNER);

    challenge_tx(template.as_json(), Error::AccountCellStatusLocked);
}

#[test]
fn challenge_sub_account_create_parent_expired() {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            },
            "data": {
                // Simulate using the AccountCell that is expired.
                "expired_at": TIMESTAMP - 1,
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let das_profit = calculate_sub_account_cost(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "data": {
                "expired_at": TIMESTAMP - 1,
            },
            "witness": {
                "enable_sub_account": 1,
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, das_profit, 0);
    push_output_normal_cell(&mut template, 10_000_000_000 - das_profit, OWNER);

    challenge_tx(template.as_json(), Error::AccountCellInExpirationGracePeriod);
}

#[test]
fn challenge_sub_account_create_parent_not_enable_feature() {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                // Simulate the parent AccountCell has not enable sub-account feature.
                "enable_sub_account": 0,
            }
        }),
    );
    push_simple_input_sub_account_cell(&mut template, 0, 0);
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let das_profit = calculate_sub_account_cost(&mut template);

    push_output_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
            },
            "witness": {
                "enable_sub_account": 0,
            }
        }),
    );
    push_simple_output_sub_account_cell(&mut template, das_profit, 0);
    push_output_normal_cell(&mut template, 10_000_000_000 - das_profit, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountFeatureNotEnabled);
}

// TODO Becasue of the issues in sparse-merkle-tree crate, SMT proof can not be generate properly in development environment, need fix.
// #[test]
// fn challenge_sub_account_create_existing_account() {
//     let mut template = init_create("create_sub_account", Some("0x00"));
//
//     // inputs
//     push_simple_input_account_cell(&mut template);
//     template.restore_sub_account(vec![json!({
//         "lock": {
//             "owner_lock_args": OWNER,
//             "manager_lock_args": MANAGER
//         },
//         "account": SUB_ACCOUNT_1,
//         "suffix": SUB_ACCOUNT_SUFFIX,
//         "registered_at": TIMESTAMP,
//         "expired_at": u64::MAX,
//     })]);
//     push_simple_input_sub_account_cell(&mut template, 0);
//     push_input_normal_cell(&mut template, 10_000_000_000, OWNER);
//
//     // outputs
//     template.push_sub_account_witness(
//         SubAccountActionType::Insert,
//         json!({
//             "sub_account": {
//                 "lock": {
//                     "owner_lock_args": OWNER_1,
//                     "manager_lock_args": MANAGER_1
//                 },
//                 "account": SUB_ACCOUNT_1,
//                 "suffix": SUB_ACCOUNT_SUFFIX,
//                 "registered_at": TIMESTAMP,
//                 "expired_at": TIMESTAMP + YEAR_SEC,
//             }
//         }),
//     );
//     push_common_output_cells(&mut template);
//
//     challenge_tx(template.as_json(), Error::SubAccountWitnessSMTRootError);
// }

#[test]
fn challenge_sub_account_create_invalid_char() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                // Simulate the sub-account contains invalid character.
                "account": "✨das🎱001.xxxxx.bit",
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::PreRegisterAccountCharIsInvalid);
}

#[test]
fn challenge_sub_account_create_undefined_char() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                // Simulate the sub-account contains undefined character.
                "account": "✨das大001.xxxxx.bit",
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::PreRegisterFoundUndefinedCharSet);
}

#[test]
fn challenge_sub_account_create_too_long() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                // Simulate the sub-account is too long.
                "account": "1234567890123456789012345678901234567890123.xxxxx.bit",
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::PreRegisterAccountIsTooLong);
}

#[test]
fn challenge_sub_account_create_suffix_not_match() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                // Simulate the suffix is not match with the parent account.
                "account": "00000.a.bit",
                "suffix": ".a.bit",
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::SubAccountInitialValueError);
}

#[test]
fn challenge_sub_account_create_id_not_match() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                // Simulate the id is not match with the account.
                "id": "0x0000000000000000000000000000000000000000",
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::SubAccountInitialValueError);
}

#[test]
fn challenge_sub_account_create_registered_at_is_invalid() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                // Simulate the registered_at is not the same as the TimeCell.
                "registered_at": TIMESTAMP - 1,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::SubAccountInitialValueError);
}

#[test]
fn challenge_sub_account_create_expired_at_less_than_one_year() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                // Simulate the expired_at is less than one year.
                "expired_at": TIMESTAMP + YEAR_SEC - 1,
            }
        }),
    );
    push_common_output_cells(&mut template);

    challenge_tx(template.as_json(), Error::SubAccountInitialValueError);
}

#[test]
fn challenge_sub_account_create_no_profit_record() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    let new_sub_account_cost = calculate_sub_account_cost(&template);
    push_simple_output_account_cell(&mut template);
    // Simulate forget record correct profit in the outputs_data of the SubAccountCell
    push_simple_output_sub_account_cell(&mut template, 0, 0);
    push_output_normal_cell(&mut template, 10_000_000_000 - new_sub_account_cost, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountProfitError);
}

#[test]
fn challenge_sub_account_create_profit_not_match_capacity() {
    let mut template = before_each();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    let new_sub_account_cost = calculate_sub_account_cost(&template);
    push_simple_output_account_cell(&mut template);

    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            // Simulate forget put profit into the capacity of the SubAccountCell
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + new_sub_account_cost - 1,
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": new_sub_account_cost
            }
        }),
    );

    push_output_normal_cell(&mut template, 10_000_000_000 - new_sub_account_cost, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountCellCapacityError);
}

#[test]
fn test_sub_account_create_with_custom_script() {
    let mut template = before_each_with_custom_script();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
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
                    "owner_lock_args": OWNER_2,
                    "manager_lock_args": MANAGER_2
                },
                "account": SUB_ACCOUNT_3,
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
                "account": SUB_ACCOUNT_4,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    push_common_output_cells_with_custom_script(&mut template);

    test_tx(template.as_json())
}

#[test]
fn challenge_sub_account_create_with_custom_script_modified_script_args() {
    let mut template = before_each_with_custom_script();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let total_profit = calculate_sub_account_custom_price(&template);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "das_profit": das_profit,
                "owner_profit": owner_profit,
                // Simulate modifying the custom script of the SubAccountCell.
                "custom_script": "0x01000000742d637573746f6d2d736372000000"
            }
        }),
    );
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountCellConsistencyError);
}

#[test]
fn challenge_sub_account_create_with_custom_script_different_lock_for_normal_cells() {
    let mut template = before_each_with_custom_script();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let total_profit = calculate_sub_account_custom_price(&template);
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit);
    // Simulate change to a different lock which is not the same as the lock in inputs.
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER_1);

    challenge_tx(template.as_json(), Error::SubAccountNormalCellLockLimit);
}

#[test]
fn challenge_sub_account_create_with_custom_script_das_profit_not_enough() {
    let mut template = before_each_with_custom_script();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let total_profit = calculate_sub_account_custom_price(&template);
    // Simulate the profit of DAS is less than expected value.
    let das_profit = total_profit * SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE / RATE_BASE - 1;
    let owner_profit = total_profit - das_profit;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit);
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountProfitError);
}

#[test]
fn challenge_sub_account_create_with_custom_script_das_profit_less_than_minimal_limit() {
    let mut template = before_each_with_custom_script();

    // outputs
    template.push_sub_account_witness(
        SubAccountActionType::Insert,
        json!({
            "sub_account": {
                "lock": {
                    "owner_lock_args": OWNER_1,
                    "manager_lock_args": MANAGER_1
                },
                "account": SUB_ACCOUNT_2,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );

    let total_profit = calculate_sub_account_cost(&template);
    // Simulate the profit of DAS is less than expected value.
    let das_profit = total_profit - 1;
    let owner_profit = 0;
    push_simple_output_sub_account_cell_with_custom_script(&mut template, das_profit, owner_profit);
    push_output_normal_cell(&mut template, 100_000_000_000 - total_profit, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountProfitError);
}
