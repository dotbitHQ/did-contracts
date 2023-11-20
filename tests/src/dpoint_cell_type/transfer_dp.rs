use dpoint_cell_type::error::ErrorCode;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init(json!({ "action": "transfer_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);

    template
}

#[test]
fn test_dpoint_transfer_dp_simple() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    test_tx(template.as_json());
}

#[test]
fn test_dpoint_transfer_dp_split() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);

    test_tx(template.as_json());
}

#[test]
fn test_dpoint_transfer_dp_merge() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    test_tx(template.as_json());
}

#[test]
fn test_dpoint_transfer_dp_whitelist_mix() {
    let mut template = init(json!({ "action": "transfer_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);
    push_input_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);
    push_input_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);

    // outputs
    push_output_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_2);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_dpoint_transfer_dp_without_transfer_whitelist_address() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 300 * USD_1, OWNER);
    // Simulate no address in transfer whitelist

    challenge_tx(template.as_json(), ErrorCode::WhitelistLockIsRequired);
}

#[test]
fn challenge_dpoint_transfer_dp_multi_user_1() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    // Simulate transfering DP to multiple users
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER_1);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::OnlyOneUserIsAllowed);
}

#[test]
fn challenge_dpoint_transfer_dp_multi_user_2() {
    let mut template = init(json!({ "action": "transfer_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    // Simulate transfering DP from multiple users
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER_1);

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::OnlyOneUserIsAllowed);
}

#[test]
fn challenge_dpoint_transfer_dp_multi_user_3() {
    let mut template = init(json!({ "action": "transfer_dp" }));

    // inputs
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_input_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    // Simulate transfering DP from multiple users
    push_input_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::OnlyOneUserIsAllowed);
}

#[test]
fn challenge_dpoint_transfer_dp_with_burn_dp() {
    let mut template = before_each();

    // outputs
    // Simulate burning some DP in outputs
    push_output_dpoint_cell(&mut template, (100 - 1) * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::TheTotalDPointCanNotChange);
}

#[test]
fn challenge_dpoint_transfer_dp_with_mint_dp() {
    let mut template = before_each();

    // outputs
    // Simulate minting some DP in outputs
    push_output_dpoint_cell(&mut template, (100 + 1) * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::TheTotalDPointCanNotChange);
}

#[test]
fn challenge_dpoint_transfer_dp_split_with_wrong_capacity() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);

    template.push_output(
        json!({
            // Simulate creating a new DPointCell without enough basic capacity
            "capacity": DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY - 1,
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

    push_output_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);

    challenge_tx(template.as_json(), ErrorCode::InitialCapacityError);
}

#[test]
fn challenge_dpoint_transfer_dp_violate_min_limit() {
    let mut template = before_each();
    // outputs
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 50 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 100 * USD_1, OWNER);

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

    push_output_dpoint_cell(&mut template, 100 * USD_1, DP_TRANSFER_WHITELIST_1);

    challenge_tx(template.as_json(), ErrorCode::InitialDataError);
}

#[test]
fn challenge_dpoint_mint_dp_invalid_capacity() {
    let mut template = before_each();

    // outputs
    template.push_output(
        json!({
            // Simulate spending too much in fee
            "capacity": DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY - 1,
            "lock": {
                "owner_lock_args": OWNER,
                "manager_lock_args": OWNER,
            },
            "type": {
                "code_hash": "{{dpoint-cell-type}}"
            },
            "data": {
                "value": 100 * USD_1
            }
        }),
        None,
    );
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);

    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        DP_RECYCLE_WHITELIST_1,
    );

    challenge_tx(template.as_json(), ErrorCode::InitialCapacityError);
}

#[test]
fn chalenge_dpoint_transfer_dp_without_recycle_whitelist_address() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 100 * USD_1, OWNER);
    push_output_dpoint_cell(&mut template, 200 * USD_1, DP_TRANSFER_WHITELIST_1);
    push_output_balance_cell(
        &mut template,
        (DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY) * 1,
        // Simulate recycling to an address not in whitelist
        CHANNEL,
    );

    challenge_tx(template.as_json(), ErrorCode::CapacityRecycleError);
}
