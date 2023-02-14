use alloc::boxed::Box;
use alloc::vec;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert as das_assert, code_to_error, debug, util, verifiers};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running reverse-record-root-cell-type ======");

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
        b"create_reverse_record_root" => {
            util::require_super_lock()?;

            parser.parse_cell()?;
            let config_reverse_resolution = parser.configs.reverse_resolution()?;

            verifiers::common::verify_cell_number_and_position(
                "ReverseRecordRootCell",
                &input_cells,
                &[],
                &output_cells,
                &[0],
            )?;

            debug!("Verify all fields of the new ReverseRecordRootCell.");

            // verify capacity
            let root_cell_capacity = high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            let expected_capacity = u64::from(config_reverse_resolution.record_basic_capacity());

            das_assert!(
                root_cell_capacity == expected_capacity,
                ReverseRecordRootCellErrorCode::InitialCapacityError,
                "The initial capacity of ReverseRecordRootCell should be equal to ConfigCellReverseResolution.record_basic_capacity .(expected: {}, current: {})",
                expected_capacity,
                root_cell_capacity
            );

            // verify lock
            verifiers::misc::verify_always_success_lock(output_cells[0], Source::Output)?;

            // verify data
            let output_data = util::load_cell_data(output_cells[0], Source::Output)?;
            das_assert!(
                output_data == vec![0u8; 32],
                ReverseRecordRootCellErrorCode::InitialOutputsDataError,
                "The initial outputs_data of ReverseRecordRootCell should be 32 bytes of 0x00."
            );
        }
        b"update_reverse_record_root" => {}
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
