use alloc::boxed::Box;
#[cfg(debug_assertions)]
use alloc::string::ToString;
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::config::Config;
use das_core::constants::{ScriptType, ONE_CKB};
use das_core::error::*;
use das_core::since_util::SinceFlag;
use das_core::{assert, code_to_error, debug, since_util, util, verifiers};
use das_types::constants::{Action, TypeScript};
use witness_parser::WitnessesParserV1;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running apply-register-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

    util::is_system_off()?;

    debug!("Route to {:?} action ...", parser.action.to_string());
    match parser.action {
        Action::ApplyRegister => {
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number_and_position(
                "ApplyRegisterCell",
                &input_cells,
                &[],
                &output_cells,
                &[0],
            )?;

            let data = util::load_cell_data(output_cells[0], Source::Output)?;

            debug!("Check if the data is a 32 bytes hash ...");

            assert!(
                data.len() == 32,
                ErrorCode::InvalidCellData,
                "The data of ApplyRegisterCell should have 32 bytes of data."
            );
        }
        Action::RefundApply => {
            let config = Config::get_instance().apply()?;
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
                refund_capacity >= expected_refund_capacity - ONE_CKB,
                ErrorCode::ApplyRegisterRefundCapacityError,
                "The total refunds should be more than {}, but {} found.",
                expected_refund_capacity - ONE_CKB,
                refund_capacity
            );

            let config_main_reader = Config::get_instance().main()?;
            verifiers::balance_cell::verify_das_lock_always_with_type(config_main_reader)?;
        }
        Action::PreRegister => {
            util::require_type_script(
                TypeScript::PreAccountCellType,
                Source::Output,
                ErrorCode::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
