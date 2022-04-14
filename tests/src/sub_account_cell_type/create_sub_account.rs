use super::common::*;
use crate::util::template_generator::SubAccountActionType;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use serde_json::json;

fn before_each() -> TemplateGenerator {
    let mut template = init_create("create_sub_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
    push_input_sub_account_cell(
        &mut template,
        json!({
            "type": {
                "args": ACCOUNT_1
            }
        }),
    );
    push_input_normal_cell(&mut template, 10_000_000_000, OWNER);

    template
}

fn push_simple_output_account_cell(template: &mut TemplateGenerator) {
    push_output_account_cell(
        template,
        json!({
            "data": {
                "account": ACCOUNT_1,
                "enable_sub_account": 1,
            }
        }),
    );
}

fn push_simple_output_sub_account_cell(template: &mut TemplateGenerator, profit: u64) {
    let current_root = template.smt_with_history.current_root();
    push_output_sub_account_cell(
        template,
        json!({
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": profit
            }
        }),
    );
}

fn push_common_output_cells(template: &mut TemplateGenerator) {
    let new_sub_account_cost = calculate_sub_account_cost(template);
    push_simple_output_account_cell(template);
    push_simple_output_sub_account_cell(template, new_sub_account_cost);
    push_output_normal_cell(template, 10_000_000_000 - new_sub_account_cost, OWNER);
}

fn calculate_sub_account_cost(template: &TemplateGenerator) -> u64 {
    SUB_ACCOUNT_NEW_PRICE * template.sub_account_outer_witnesses.len() as u64
}

// push_output_normal_cell(template, 10_000_000_000 - new_sub_account_cost, OWNER);

#[test]
fn test_sub_account_create() {
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
                    "owner_lock_args": OWNER_3,
                    "manager_lock_args": MANAGER_3
                },
                "account": SUB_ACCOUNT_1,
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
                "account": SUB_ACCOUNT_1,
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
                "account": SUB_ACCOUNT_1,
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
                "account": SUB_ACCOUNT_1,
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
                "account": SUB_ACCOUNT_1,
                "suffix": SUB_ACCOUNT_SUFFIX,
                "registered_at": TIMESTAMP,
                "expired_at": TIMESTAMP + YEAR_SEC,
            }
        }),
    );
    let new_sub_account_cost = calculate_sub_account_cost(&template);
    push_simple_output_account_cell(&mut template);
    // Simulate forget record correct profit in the outputs_data of the SubAccountCell
    push_simple_output_sub_account_cell(&mut template, 0);
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
                "account": SUB_ACCOUNT_1,
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
            "capacity": SUB_ACCOUNT_BASIC_CAPACITY + SUB_ACCOUNT_PREPARED_FEE_CAPACITY,
            "type": {
                "args": ACCOUNT_1
            },
            "data": {
                "root": String::from("0x") + &hex::encode(&current_root),
                "profit": new_sub_account_cost
            }
        }),
    );

    push_output_normal_cell(&mut template, 10_000_000_000 - new_sub_account_cost, OWNER);

    challenge_tx(template.as_json(), Error::SubAccountCellCapacityError);
}
