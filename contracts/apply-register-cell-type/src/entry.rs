use alloc::boxed::Box;
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::{OracleCellType, ScriptType, TypeScript};
use das_core::error::*;
use das_core::since_util::SinceFlag;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, code_to_error, data_parser, debug, since_util, util, verifiers};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running apply-register-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );
    match action {
        b"apply_register" => {
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("ApplyRegisterCell", &input_cells, 0, &output_cells, 1)?;

            let data = util::load_cell_data(output_cells[0], Source::Output)?;

            debug!("Check if the first 32 bytes exists ...");

            assert!(
                data.len() == 48,
                ErrorCode::InvalidCellData,
                "The data of ApplyRegisterCell should have 48 bytes of data."
            );

            debug!("Check if the ApplyRegisterCell.data.height is match with the HeightCell.data.height ...");

            let apply_height = data_parser::apply_register_cell::get_height(&data);
            let expected_height = util::load_oracle_data(OracleCellType::Height)?;
            assert!(
                apply_height == expected_height,
                ErrorCode::InvalidCellData,
                "The block number in ApplyRegisterCell data should be the same as which in HeightCell."
            );

            debug!("Check if the ApplyRegisterCell.data.timestamp is match with the HeightCell.data.timestamp ...");

            let apply_time = data_parser::apply_register_cell::get_timestamp(&data);
            let expected_time = util::load_oracle_data(OracleCellType::Time)?;
            assert!(
                apply_time == expected_time,
                ErrorCode::InvalidCellData,
                "The timestamp in ApplyRegisterCell data should be the same as which in TimeCell."
            );
        }
        b"refund_apply" => {
            let config = parser.configs.apply()?;
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

            verifiers::common::verify_cell_number_range(
                "ApplyRegisterCell",
                &input_cells,
                (Ordering::Greater, 0),
                &output_cells,
                (Ordering::Equal, 0),
            )?;

            let max_waiting_block_number = u32::from(config.apply_max_waiting_block_number()) as u64;
            let mut expected_since = 0u64;
            expected_since = since_util::set_relative_flag(expected_since, SinceFlag::Relative);
            expected_since = since_util::set_metric_flag(expected_since, SinceFlag::Height);
            expected_since = since_util::set_value(expected_since, max_waiting_block_number);

            debug!("Check if the lock and since field of all ApplyRegisterCells in inputs ...");

            let expected_lock_script = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let mut expected_refund_capacity = 0;
            for index in input_cells {
                let lock_script = high_level::load_cell_lock(index, Source::Input)?;
                assert!(
                    util::is_entity_eq(&lock_script, &expected_lock_script),
                    ErrorCode::ApplyLockMustBeUnique,
                    "The lock script of all ApplyRegisterCells in inputs should be the same."
                );

                let since = high_level::load_input_since(index, Source::Input)?;
                assert!(
                    expected_since == since,
                    ErrorCode::ApplyRegisterSinceMismatch,
                    "inputs[{}] The since of ApplyRegisterCell is not correct.(expected: {}, current: {})",
                    index,
                    expected_since,
                    since
                );

                expected_refund_capacity += high_level::load_cell_capacity(index, Source::Input)?;
            }

            debug!("Check if the capacity of refund is correct ...");

            let refund_cells = util::find_cells_by_script(
                ScriptType::Lock,
                expected_lock_script.as_reader().into(),
                Source::Output,
            )?;

            let mut refund_capacity = 0;
            for index in refund_cells {
                refund_capacity += high_level::load_cell_capacity(index, Source::Output)?;
            }

            assert!(
                refund_capacity >= expected_refund_capacity - 100_000_000,
                ErrorCode::ApplyRegisterRefundCapacityError,
                "The total refunds should be more than {}, but {} found.",
                expected_refund_capacity - 100_000_000,
                refund_capacity
            );

            let config_main_reader = parser.configs.main()?;
            verifiers::balance_cell::verify_das_lock_always_with_type(config_main_reader)?;
        }
        b"pre_register" => {
            util::require_type_script(
                &parser,
                TypeScript::PreAccountCellType,
                Source::Output,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
