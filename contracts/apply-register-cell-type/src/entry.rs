use ckb_std::{ckb_constants::Source, high_level};
use core::result::Result;
use das_core::{
    assert,
    constants::{OracleCellType, ScriptType, TypeScript},
    data_parser, debug,
    error::Error,
    util, verifiers,
    witness_parser::WitnessesParser,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running apply-register-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );
    match action {
        b"apply_register" => {
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("ApplyRegisterCell", &input_cells, 0, &output_cells, 1)?;

            let data = util::load_cell_data(output_cells[0], Source::Output)?;

            debug!("Check if the first 32 bytes exists ...");

            assert!(
                data.len() == 48,
                Error::InvalidCellData,
                "The data of ApplyRegisterCell should have 48 bytes of data."
            );

            debug!("Check if the ApplyRegisterCell.data.height is match with the HeightCell.data.height ...");

            let apply_height = data_parser::apply_register_cell::get_height(&data);
            let expected_height = util::load_oracle_data(OracleCellType::Height)?;
            assert!(
                apply_height == expected_height,
                Error::InvalidCellData,
                "The block number in ApplyRegisterCell data should be the same as which in HeightCell."
            );

            debug!("Check if the ApplyRegisterCell.data.timestamp is match with the HeightCell.data.timestamp ...");

            let apply_time = data_parser::apply_register_cell::get_timestamp(&data);
            let expected_time = util::load_oracle_data(OracleCellType::Time)?;
            assert!(
                apply_time == expected_time,
                Error::InvalidCellData,
                "The timestamp in ApplyRegisterCell data should be the same as which in TimeCell."
            );
        }
        b"refund_apply" => {
            let config = parser.configs.apply()?;

            // Find out ApplyRegisterCells in current transaction.
            let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
            verifiers::common::verify_cell_number("ApplyRegisterCell", &input_cells, 1, &output_cells, 0)?;

            debug!("Check if the ApplyRegisterCell is available for refund ...");

            let data = util::load_cell_data(input_cells[0], Source::Input)?;

            assert!(
                data.len() == 48,
                Error::InvalidCellData,
                "The data of ApplyRegisterCell should have 48 bytes of data."
            );

            // Then follows the 8 bytes u64.
            let apply_height = data_parser::apply_register_cell::get_height(&data);
            let max_waiting_block_number = u32::from(config.apply_max_waiting_block_number()) as u64;

            let current_height = util::load_oracle_data(OracleCellType::Height)?;
            assert!(
                apply_height + max_waiting_block_number < current_height,
                Error::ApplyRegisterRefundNeedWaitLonger,
                "The ApplyRegisterCell can be refunded only if it has passed {} blocks since it created.(created_height: {}, current_height: {})",
                max_waiting_block_number,
                apply_height,
                current_height
            );

            debug!("Check if the capacity of refund is correct ...");

            let lock_script = high_level::load_cell_lock(input_cells[0], Source::Input).map_err(|e| Error::from(e))?;
            let transfer_cells = util::find_cells_by_script(ScriptType::Lock, lock_script.as_reader(), Source::Output)?;
            assert!(
                transfer_cells.len() == 1,
                Error::InvalidTransactionStructure,
                "There should be one cell in outputs which refund the capacity of the ApplyRegisterCell."
            );

            let expected_capacity =
                high_level::load_cell_capacity(input_cells[0], Source::Input).map_err(|e| Error::from(e))?;
            let transferred_capacity =
                high_level::load_cell_capacity(transfer_cells[0], Source::Output).map_err(|e| Error::from(e))?;
            assert!(
                transferred_capacity >= expected_capacity - 100_000_000,
                Error::ApplyRegisterRefundCapacityError,
                "The refund of the ApplyRegisterCell should be more than {}, but {} found.",
                expected_capacity - 100_000_000,
                transferred_capacity
            );
        }
        b"pre_register" => {
            util::require_type_script(
                &parser,
                TypeScript::PreAccountCellType,
                Source::Output,
                Error::InvalidTransactionStructure,
            )?;
        }
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}
