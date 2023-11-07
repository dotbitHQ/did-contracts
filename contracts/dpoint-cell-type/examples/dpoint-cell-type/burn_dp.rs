use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::TypeScript;
use das_core::contract::defult_structs::{Action, Rule};
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, das_assert, debug, util as core_util};
use das_types::packed::*;
use dpoint_cell_type::error::ErrorCode;

pub fn action() -> Result<Action, Box<dyn ScriptError>> {
    let parser = WitnessesParser::new()?;
    core_util::is_system_off(&parser)?;

    let config_dpoint_reader = parser.configs.dpoint()?;

    let mut action = Action::new("burn_dp");

    let (input_cells, output_cells) = core_util::load_self_cells_in_inputs_and_outputs()?;

    let inner_input_cells = input_cells.clone();
    let inner_output_cells = output_cells.clone();
    action.add_verification(Rule::new("Verify the transaction structure.", move |_contract| {
        das_assert!(
            inner_input_cells.len() > 0 && inner_input_cells.len() >= inner_output_cells.len(),
            ErrorCode::InvalidTransactionStructure,
            "There should be more than 0 DPointCells in the inputs, and creating new DPointCells is not allowed."
        );

        let mut owner_lock = None;
        for i in inner_input_cells.iter() {
            let lock = high_level::load_cell_lock(*i, Source::Input)?;
            if owner_lock.is_none() {
                owner_lock = Some(lock);
            } else {
                das_assert!(
                    core_util::is_entity_eq(owner_lock.as_ref().unwrap(), &lock),
                    ErrorCode::OnlyOneUserIsAllowed,
                    "inputs[{}] The owner of DPointCell should be the same.",
                    i
                );
            }
        }

        if inner_output_cells.len() > 0 {
            for i in inner_output_cells.iter() {
                let lock = high_level::load_cell_lock(*i, Source::Output)?;
                das_assert!(
                    core_util::is_entity_eq(owner_lock.as_ref().unwrap(), &lock),
                    ErrorCode::OnlyOneUserIsAllowed,
                    "outputs[{}] The owner of DPointCell should be the same.",
                    i
                );
            }
        }

        Ok(())
    }));

    let input_dpoint_cells = input_cells.clone();
    let output_dpoint_cells = output_cells.clone();
    action.add_verification(Rule::new("Verify the DPoints is decreased.", move |_contract| {
        let total_input_dp = core_util::get_total_dpoint(&input_dpoint_cells, Source::Input)?;
        let total_output_dp = core_util::get_total_dpoint(&output_dpoint_cells, Source::Output)?;
        das_assert!(
            total_input_dp > total_output_dp,
            ErrorCode::TheDPointShouldDecreased,
            "The total DPoint in output should be decreased."
        );

        Ok(())
    }));

    let transfer_whitelist = config_dpoint_reader.transfer_whitelist();
    let transfer_whitelist_hashes = transfer_whitelist
        .iter()
        .map(|lock| core_util::blake2b_256(lock.as_slice()))
        .collect::<Vec<_>>();
    action.add_verification(Rule::new(
        "Verify if there is any address in inputs exist in whitelist.",
        move |_contract| {
            let mut has_whitelist_lock = false;

            let mut i = 0;
            loop {
                let ret = high_level::load_cell_lock_hash(i, Source::Input);
                match ret {
                    Ok(lock_hash) => {
                        if transfer_whitelist_hashes.contains(&lock_hash) {
                            has_whitelist_lock = true;
                            break;
                        }
                    }
                    Err(_) => break,
                }

                i += 1;
            }

            das_assert!(
                has_whitelist_lock,
                ErrorCode::WhitelistLockIsRequired,
                "Only the lock in whitelist can push this transaction."
            );

            Ok(())
        },
    ));

    let mut recycle_capacity = 0;
    if input_cells.len() > output_cells.len() {
        let start = output_cells.len();
        for index in input_cells[start..].iter() {
            let capacity = high_level::load_cell_capacity(*index, Source::Input)?;
            recycle_capacity += capacity;
        }
    }

    // TODO load this value from new ConfigCellMain
    let common_fee = 20000;

    if output_cells.len() > 0 {
        let inner_input_cells = input_cells.clone();
        let inner_output_cells = output_cells.clone();
        action.add_verification(Rule::new(
            "Verify if the remaining DPointCell is charged for fee properly.",
            move |_contract| {
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
                            das_assert!(
                                capacity + common_fee >= *input_capacity,
                                ErrorCode::SpendTooMuchFee,
                                "outputs[{}] The capacity of DPointCell spent more than the fee limit.",
                                index
                            );
                        }
                        None => unreachable!(),
                    }

                    total_output += capacity;
                }

                das_assert!(
                    total_output + common_fee >= total_input - recycle_capacity,
                    ErrorCode::SpendTooMuchFee,
                    "The total capacity of outputs spent more than the fee limit."
                );

                Ok(())
            },
        ));
    }

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
    } else {
        debug!("Skip verifying if the DPointCells' capacity is recycled properly.")
    }

    action.add_verification(Rule::new("Verify the EIP712 signature.", move |_contract| {
        core_util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        Ok(())
    }));

    Ok(action)
}
