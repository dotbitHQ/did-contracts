use dpoint_cell_type::error::ErrorCode;
use serde_json::json;

use super::common::*;
use crate::util::accounts::*;
use crate::util::constants::*;
use crate::util::template_common_cell::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn before_each() -> TemplateGenerator {
    let mut template = init(json!({ "action": "mint_dp" }));

    // inputs
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    template
}

#[test]
fn test_dpoint_mint_dp() {
    let mut template = before_each();

    // outputs
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_2);
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_3);

    test_tx(template.as_json());
}

#[test]
fn challenge_dpoint_mint_dp_without_super_lock() {
    let mut template = init(json!({ "action": "mint_dp" }));

    // inputs
    // Simulate minting DP without super lock
    push_input_normal_cell(&mut template, 0, CHANNEL);

    // outputs
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_1);

    challenge_tx(template.as_json(), ErrorCode::SuperLockIsRequired);
}

#[test]
fn challenge_dpoint_mint_dp_to_invalid_owner() {
    let mut template = before_each();

    // outputs
    // Simulate minting DP to an address now in whitelist
    push_output_dpoint_cell(&mut template, 100, CHANNEL);

    challenge_tx(template.as_json(), ErrorCode::InitialOwnerError);
}

#[test]
fn challenge_dpoint_mint_dp_with_burn_dp() {
    let mut template = init(json!({ "action": "mint_dp" }));

    // inputs
    // Simulate burning DP in this action
    push_input_dpoint_cell(&mut template, 50, DP_TRANSFER_WHITELIST_1);
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_1);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure);
}

#[test]
fn challenge_dpoint_mint_dp_with_spend_dp() {
    let mut template = init(json!({ "action": "mint_dp" }));

    // inputs
    // Simulate spending DP in this action
    push_input_dpoint_cell(&mut template, 50, DP_TRANSFER_WHITELIST_1);
    push_input_normal_cell(&mut template, 0, SUPER_LOCK_ARGS);

    // outputs
    push_input_dpoint_cell(&mut template, 50, DP_TRANSFER_WHITELIST_1);
    push_output_dpoint_cell(&mut template, 100, DP_TRANSFER_WHITELIST_1);

    challenge_tx(template.as_json(), ErrorCode::InvalidTransactionStructure);
}

#[test]
fn challenge_dpoint_mint_dp_invalid_capacity() {
    let mut template = before_each();
    // outputs
    template.push_output(
        json!({
            // Simulate not providing enough capacity for the DPointCell
            "capacity": DPOINT_BASIC_CAPACITY + DPOINT_PREPARED_FEE_CAPACITY - 1,
            "lock": {
                "owner_lock_args": DP_TRANSFER_WHITELIST_1,
                "manager_lock_args": DP_TRANSFER_WHITELIST_1,
            },
            "type": {
                "code_hash": "{{dpoint-cell-type}}"
            },
            "data": {
                "value": 100 * USD_1,
            }
        }),
        None,
    );

    challenge_tx(template.as_json(), ErrorCode::InitialCapacityError);
}
