use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::{ScriptType, DPOINT_MAX_LIMIT};
use das_core::contract::defult_structs::{Action, Rule};
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, das_assert, data_parser, util as core_util, verifiers};
use das_types::constants::super_lock;
use das_types::packed::*;
use dpoint_cell_type::error::ErrorCode;

pub fn action() -> Result<Action, Box<dyn ScriptError>> {
    let parser = WitnessesParser::new()?;
    core_util::is_system_off(&parser)?;

    let config_dpoint_reader = parser.configs.dpoint()?;

    let mut action = Action::new("mint_dp");
    let (input_cells, output_cells) = core_util::load_self_cells_in_inputs_and_outputs()?;

    let inner_input_cells = input_cells.clone();
    let inner_output_cells = output_cells.clone();
    action.add_verification(Rule::new("Verify the transaction structure.", move |_contract| {
        verifiers::common::verify_cell_number_range(
            "DPointCell",
            &inner_input_cells,
            (Ordering::Equal, 0),
            &inner_output_cells,
            (Ordering::Greater, 0),
        )?;
        Ok(())
    }));

    action.add_verification(Rule::new(
        "Verify if the inputs containing cells with super lock.",
        move |_contract| {
            let expected_lock = super_lock();
            let cells =
                core_util::find_cells_by_script(ScriptType::Lock, expected_lock.as_reader().into(), Source::Input)?;

            das_assert!(
                cells.len() > 0,
                ErrorCode::SuperLockIsRequired,
                "The super lock is required in inputs."
            );

            Ok(())
        },
    ));

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

    let inner_output_cells = output_cells.clone();
    let transfer_whitelist = config_dpoint_reader.transfer_whitelist();
    let transfer_whitelist_hashes = transfer_whitelist
        .iter()
        .map(|lock| core_util::blake2b_256(lock.as_slice()))
        .collect::<Vec<_>>();
    action.add_verification(Rule::new("Verify if all the DPointCells transfered to addresses in whitelist.", move |_contract| {
        for index in inner_output_cells.iter() {
            let lock_hash = high_level::load_cell_lock_hash(*index, Source::Output)?;
            das_assert!(
                transfer_whitelist_hashes.contains(&lock_hash),
                ErrorCode::InitialOwnerError,
                "outputs[{}] The lock of new DPointCell is wrong, the lock should be in the ConfigCellDPoint.transfer_whitelist .",
                index
            )
        }

        Ok(())
    }));

    Ok(action)
}
