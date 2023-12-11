use alloc::boxed::Box;
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, code_to_error, debug, util, verifiers};
use das_types::constants::TypeScript;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running reverse-record-cell-type ======");

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

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    match action {
        b"retract_reverse_record" => {
            let config_main = parser.configs.main()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

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

            util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;
        }
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
