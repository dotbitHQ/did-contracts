use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::TypeScript;
use das_core::contract::defult_structs::{Action, Rule};
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, das_assert, util as core_util, verifiers};
use das_types::packed::*;
use dpoint_cell_type::error::ErrorCode;

use super::util;

pub fn action() -> Result<Action, Box<dyn ScriptError>> {
    let mut parser = WitnessesParser::new()?;
    let witness_action = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };

    core_util::is_system_off(&parser)?;

    let config_dpoint_reader = parser.configs.dpoint()?;

    let mut action = Action::new("transfer_dp");
    action.is_default = true;

    let (input_cells, output_cells) = core_util::load_self_cells_in_inputs_and_outputs()?;
    let grouped_input_cells = util::group_cells_by_lock(&input_cells, Source::Input)?;
    let grouped_output_cells = util::group_cells_by_lock(&output_cells, Source::Output)?;
    let transfer_whitelist = config_dpoint_reader.transfer_whitelist();
    let transfer_whitelist_hashes = transfer_whitelist
        .iter()
        .map(|lock| core_util::blake2b_256(lock.as_slice()))
        .collect::<Vec<_>>();

    let inner_input_cells = input_cells.clone();
    let inner_output_cells = output_cells.clone();
    let inner_grouped_input_cells = grouped_input_cells.clone();
    let inner_grouped_output_cells = grouped_output_cells.clone();
    let inner_expected_lock_hashes = transfer_whitelist_hashes.clone();
    action.add_verification(Rule::new("Verify the transaction structure.", move |_contract| {
        verifiers::common::verify_cell_number_range(
            "DPointCell",
            &inner_input_cells,
            (Ordering::Greater, 0),
            &inner_output_cells,
            (Ordering::Greater, 0),
        )?;

        let input_user_group_locks: Vec<&[u8; 32]> = inner_grouped_input_cells
            .iter()
            .filter(|(key, _)| !inner_expected_lock_hashes.contains(key))
            .map(|(key, _)| key)
            .collect();
        let output_user_group_locks: Vec<&[u8; 32]> = inner_grouped_output_cells
            .iter()
            .filter(|(key, _)| !inner_expected_lock_hashes.contains(key))
            .map(|(key, _)| key)
            .collect();

        // Covered cases:
        // Server in whitelist -> User
        // User -> Server in whitelist
        // Server in whitelist -> Server in whitelist
        das_assert!(
            input_user_group_locks.len() <= 1 && output_user_group_locks.len() <= 1,
            ErrorCode::OnlyOneUserIsAllowed,
            "Only one owner is allowed in each transfer."
        );

        if input_user_group_locks.len() == 1 && output_user_group_locks.len() == 1 {
            das_assert!(
                input_user_group_locks[0] == output_user_group_locks[0],
                ErrorCode::OnlyOneUserIsAllowed,
                "The owner in inputs and outputs should be the same."
            );
        }

        Ok(())
    }));

    let inner_grouped_input_cells = grouped_input_cells.clone();
    let inner_grouped_output_cells = grouped_output_cells.clone();
    let inner_expected_lock_hashes = transfer_whitelist_hashes.clone();
    action.add_verification(Rule::new(
        "Verify if there is any transfer address on whitelist.",
        move |_contract| {
            let has_lock_in_whitelist;

            let whitelist_lock_count = inner_grouped_input_cells
                .iter()
                .filter(|(key, _)| inner_expected_lock_hashes.contains(key))
                .count();

            if whitelist_lock_count > 0 {
                has_lock_in_whitelist = true;
            } else {
                let whitelist_lock_count = inner_grouped_output_cells
                    .iter()
                    .filter(|(key, _)| inner_expected_lock_hashes.contains(key))
                    .count();
                has_lock_in_whitelist = whitelist_lock_count > 0;
            }

            das_assert!(
                has_lock_in_whitelist,
                ErrorCode::WhitelistLockIsRequired,
                "There should be some lock in the transfer whitelist join the transaction."
            );

            Ok(())
        },
    ));

    let inner_input_cells = input_cells.clone();
    let inner_output_cells = output_cells.clone();
    action.add_verification(Rule::new("Verify the DPoints is not decreased.", move |_contract| {
        let total_input_dp = core_util::get_total_dpoint(&inner_input_cells, Source::Input)?;
        let total_output_dp = core_util::get_total_dpoint(&inner_output_cells, Source::Output)?;
        das_assert!(
            total_input_dp == total_output_dp,
            ErrorCode::TheDPointCanNotDecreased,
            "The total input DPoint should be equal to the total output DPoint."
        );

        Ok(())
    }));

    let basic_capacity = u64::from(config_dpoint_reader.basic_capacity());
    let prepared_fee_capacity = u64::from(config_dpoint_reader.prepared_fee_capacity());
    let expected_capacity = basic_capacity + prepared_fee_capacity;
    let mut recycle_capacity = 0;
    let mut fill_capacity = 0;
    if input_cells.len() > output_cells.len() {
        let start = output_cells.len();
        for index in input_cells[start..].iter() {
            let capacity = high_level::load_cell_capacity(*index, Source::Input)?;
            recycle_capacity += capacity;
        }
    } else if input_cells.len() < output_cells.len() {
        let count = output_cells.len() - input_cells.len();
        fill_capacity = (basic_capacity + prepared_fee_capacity) * count as u64;
    }
    // TODO load this value from new ConfigCellMain
    let common_fee = 20000;

    let inner_witness_action = witness_action.clone();
    let inner_input_cells = input_cells.clone();
    let inner_output_cells = output_cells.clone();
    action.add_verification(Rule::new(
        "Verify if the transaction fee spent properly.",
        move |_contract| {
            let can_spend_fee = inner_witness_action.as_slice() == b"transfer_dp";

            let mut input_capacities = vec![];
            let mut total_input = 0;
            for index in inner_input_cells.iter() {
                let capacity = high_level::load_cell_capacity(*index, Source::Input)?;
                input_capacities.push(capacity);
                total_input += capacity;
            }

            let mut total_output = 0;
            for (i, index) in inner_output_cells.iter().enumerate() {
                let capacity = high_level::load_cell_capacity(*index, Source::Output)?;

                match input_capacities.get(i) {
                    Some(input_capacity) => {
                        if can_spend_fee {
                            das_assert!(
                                capacity + common_fee >= *input_capacity,
                                ErrorCode::SpendTooMuchFee,
                                "outputs[{}] The capacity of DPointCell spent more than the fee limit.",
                                index
                            );
                        } else {
                            das_assert!(
                                capacity == *input_capacity,
                                ErrorCode::CanNotSpendAnyFee,
                                "outputs[{}] The capacity of DPointCell should be equal to the input.",
                                index
                            );
                        }
                    },
                    None => {
                        das_assert!(
                            capacity == expected_capacity,
                            ErrorCode::InitialCapacityError,
                            "outputs[{}] The capacity of new DPointCell should be {} shannon.(expected: {}, current: {})",
                            index,
                            expected_capacity,
                            expected_capacity,
                            capacity
                        );
                    }
                }

                total_output += capacity;
            }

            if can_spend_fee {
                das_assert!(
                    total_output - fill_capacity  + common_fee >= total_input - recycle_capacity,
                    ErrorCode::SpendTooMuchFee,
                    "The total capacity of outputs spent more than the fee limit."
                );
            } else {
                das_assert!(
                    total_output - fill_capacity >= total_input - recycle_capacity,
                    ErrorCode::CanNotSpendAnyFee,
                    "This transaction need to pay the fee from Other cells."
                );
            }

            Ok(())
        },
    ));

    if recycle_capacity > 0 {
        let recycle_whitelist = config_dpoint_reader.capacity_recycle_whitelist();
        let recycle_whitelist_hashes = recycle_whitelist
            .iter()
            .map(|lock| core_util::blake2b_256(lock.as_slice()))
            .collect::<Vec<_>>();
        action.add_verification(Rule::new(
            "Verify if the DPoints' capacity recycled properly.",
            move |_contract| {
                let mut actual_recycle = 0;
                let mut i = 0;
                loop {
                    let ret = high_level::load_cell_lock_hash(i, Source::Output);
                    match ret {
                        Ok(lock_hash) => {
                            if recycle_whitelist_hashes.contains(&lock_hash) {
                                let capacity = high_level::load_cell_capacity(i, Source::Output)?;
                                actual_recycle += capacity;
                            }
                        }
                        Err(_) => break,
                    }

                    i += 1;
                }

                das_assert!(
                    actual_recycle >= recycle_capacity - common_fee,
                    ErrorCode::CapacityRecycleError,
                    "The total capacity should be recycled is {}.(expected: {}, actual: {})",
                    recycle_capacity,
                    recycle_capacity,
                    actual_recycle
                );

                Ok(())
            },
        ));
    }

    if witness_action.as_slice() == b"transfer_dp" {
        action.add_verification(Rule::new("Verify the EIP712 signature.", move |_contract| {
            core_util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
            Ok(())
        }));
    }

    Ok(action)
}
