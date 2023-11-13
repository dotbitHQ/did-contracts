use dpoint_cell_type::error::ErrorCode;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init(json!({ "action": "burn_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100, OWNER);
    push_input_dpoint_cell(&mut template, 100, OWNER);
    push_input_dpoint_cell(&mut template, 100, OWNER);
    push_input_balance_cell(&mut template, 0, DP_RECYCLE_WHITELIST_1);

    template
}

#[test]
fn test_dpoint_burn_dp_simple() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    test_tx(template.as_json());
}

#[test]
fn test_dpoint_burn_dp_merge() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 200, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    test_tx(template.as_json());
}

#[test]
fn test_dpoint_burn_dp_split() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_dpoint_burn_dp_without_any_whitelist_address() {
    let mut template = init(json!({ "action": "burn_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100, OWNER);
    push_input_dpoint_cell(&mut template, 100, OWNER);
    push_input_dpoint_cell(&mut template, 100, OWNER);
    // Simulate burning DP without whitelist address
    push_input_balance_cell(&mut template, 0, CHANNEL);

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::WhitelistLockIsRequired);
}

#[test]
fn challenge_dpoint_burn_dp_multiple_owner_1() {
    let mut template = init(json!({ "action": "burn_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100, OWNER_1);
    push_input_dpoint_cell(&mut template, 100, OWNER_2);
    push_input_dpoint_cell(&mut template, 100, OWNER_3);
    // Simulate burning DP without whitelist address
    push_input_balance_cell(&mut template, 0, CHANNEL);

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER_1);
    push_output_dpoint_cell(&mut template, 50, OWNER_2);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::OnlyOneUserIsAllowed);
}

#[test]
fn challenge_dpoint_burn_dp_multiple_owner_2() {
    let mut template = init(json!({ "action": "burn_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100, OWNER_1);
    push_input_dpoint_cell(&mut template, 100, OWNER_1);
    push_input_dpoint_cell(&mut template, 100, OWNER_1);
    // Simulate burning DP without whitelist address
    push_input_balance_cell(&mut template, 0, CHANNEL);

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER_1);
    push_output_dpoint_cell(&mut template, 50, OWNER_2);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::OnlyOneUserIsAllowed);
}

#[test]
fn challenge_dpoint_burn_dp_violate_min_limit() {
    let mut template = before_each();
    // outputs
    template.push_output(
        json!({
            "capacity": DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY,
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": OWNER,
            },
            "type": {
                "code_hash": "{{dpoint-cell-type}}"
            },
            "data": {
                // Simulate providing a invalid value
                "value": 0
            }
        }),
        None,
    );

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::InitialDataError);
}

#[test]
fn challenge_dpoint_burn_dp_value_increased() {
    let mut template = before_each();

    // outputs
    // Simulate increasing the DPoint in the outputs
    push_output_dpoint_cell(&mut template, 300 + 1, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::TheDPointShouldDecreased);
}

#[test]
fn challenge_dpoint_burn_dp_spend_too_much_fee_in_one_cell() {
    let mut template = before_each();

    // outputs
    template.push_output(
        json!({
            // Simulate spending too much in fee
            "capacity": DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY - FEE - 1,
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": OWNER,
            },
            "type": {
                "code_hash": "{{dpoint-cell-type}}"
            },
            "data": {
                "value": 50 * USD_1
            }
        }),
        None,
    );

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::SpendTooMuchFee);
}

#[test]
fn challenge_dpoint_burn_dp_without_recycle_whitelist_address() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2,
        // Simulate recycling to an address not in whitelist
        CHANNEL,
    );

    challenge_tx(template.as_json(), ErrorCode::CapacityRecycleError);
}

#[test]
fn challenge_dpoint_burn_dp_recycle_capacity_not_enough() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50, OWNER);
    push_output_balance_cell(
        &mut template,
        // Simulate spending too much in fee
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 2 - FEE - 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::CapacityRecycleError);
}
