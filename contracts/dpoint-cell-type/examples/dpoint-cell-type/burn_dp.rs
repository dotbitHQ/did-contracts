use alloc::boxed::Box;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::DPOINT_MAX_LIMIT;
use das_core::contract::defult_structs::{Action, Rule};
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, das_assert, data_parser, debug, util as core_util};
use das_types::constants::TypeScript;
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
            inner_input_cells.len() > 0,
            ErrorCode::InvalidTransactionStructure,
            "There should be more than 0 DPointCells in the inputs."
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

    let inner_output_cells = output_cells.clone();
    let basic_capacity = u64::from(config_dpoint_reader.basic_capacity());
    let prepared_fee_capacity = u64::from(config_dpoint_reader.prepared_fee_capacity());
    let expected_capacity = basic_capacity + prepared_fee_capacity;
    action.add_verification(Rule::new(
        "Verify if all the DPointCells keeping enough capacity.",
        move |_contract| {
            for index in inner_output_cells.iter() {
                let capacity = high_level::load_cell_capacity(*index, Source::Output)?;
                das_assert!(
                    capacity == expected_capacity,
                    ErrorCode::InitialCapacityError,
                    "outputs[{}] The capacity of new DPointCell should be {} shannon.(expected: {}, current: {})",
                    index,
                    expected_capacity,
                    expected_capacity,
                    capacity
                )
            }

            Ok(())
        },
    ));

    let inner_output_cells = output_cells.clone();
    action.add_verification(Rule::new(
        "Verify if all the DPointCells has valid data.",
        move |_contract| {
            for index in inner_output_cells.iter() {
                let data = high_level::load_cell_data(*index, Source::Output)?;
                let value = data_parser::dpoint_cell::get_value(&data);

                das_assert!(
                    value.is_some(),
                    ErrorCode::InitialDataError,
                    "outputs[{}] The value of new DPointCell should be some LV structure u64 data.(current: {})",
                    index,
                    core_util::hex_string(&data)
                );

                let value = value.unwrap();
                das_assert!(
                    value > 0 && value <= DPOINT_MAX_LIMIT,
                    ErrorCode::InitialDataError,
                    "outputs[{}] The value of each new DPointCell should be 0 < x <= {}.(current: {})",
                    index,
                    DPOINT_MAX_LIMIT,
                    value
                );
            }

            Ok(())
        },
    ));

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
    let recycle_whitelist = config_dpoint_reader.capacity_recycle_whitelist();
    let recycle_whitelist_hashes = recycle_whitelist
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
                        if transfer_whitelist_hashes.contains(&lock_hash)
                            || recycle_whitelist_hashes.contains(&lock_hash)
                        {
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

    if input_cells.len() > output_cells.len() {
        let mut recycle_capacity = 0;
        let start = output_cells.len();
        for index in input_cells[start..].iter() {
            let capacity = high_level::load_cell_capacity(*index, Source::Input)?;
            recycle_capacity += capacity;
        }

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
                    actual_recycle >= recycle_capacity,
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
