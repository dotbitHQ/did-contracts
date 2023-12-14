use alloc::boxed::Box;
#[cfg(debug_assertions)]
use alloc::string::ToString;
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::config::Config;
use das_core::error::*;
use das_core::{assert, code_to_error, debug, util, verifiers};
use das_types::constants::{Action, TypeScript};
use witness_parser::WitnessesParserV1;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running reverse-record-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

    util::is_system_off()?;

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

    debug!("Route to {:?} action ...", parser.action.to_string());
    match parser.action {
        Action::RetractReverseRecord => {
            let config_main = Config::get_instance().main()?;
            let config_reverse_resolution = Config::get_instance().reverse_resolution()?;

            verifiers::common::verify_cell_number_range(
                "ReverseRecordCell",
                &input_cells,
                (Ordering::Greater, 0),
                &output_cells,
                (Ordering::Equal, 0),
            )?;

            verifiers::misc::verify_no_more_cells(&input_cells, Source::Input)?;

            debug!(
                "Verify if all ReverseRecordCells in inputs has the same lock script with the first ReverseRecordCell."
            );

            let expected_lock_hash = high_level::load_cell_lock_hash(input_cells[0], Source::Input)?;
            let mut total_input_capacity = 0;
            for i in input_cells.iter() {
                let lock_hash = high_level::load_cell_lock_hash(*i, Source::Input)?;
                assert!(
                    expected_lock_hash == lock_hash,
                    ErrorCode::InvalidTransactionStructure,
                    "Inputs[{}] The ReverseRecordCell should has the same lock script with others.",
                    i
                );

                // CAREFUL, ensure that the total input capacity is calculated from real cells in inputs, because the ReverseRecordCells' capacity is dynamic.
                total_input_capacity += high_level::load_cell_capacity(*i, Source::Input)?;
            }

            debug!("Verify if all capacity have been refund to user correctly.");

            let expected_lock = high_level::load_cell_lock(input_cells[0], Source::Input)?;
            let common_fee = u64::from(config_reverse_resolution.common_fee());
            verifiers::misc::verify_user_get_change(
                config_main,
                expected_lock.as_reader(),
                total_input_capacity - common_fee,
            )?;

            util::exec_by_type_id(TypeScript::EIP712Lib, &[])?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
