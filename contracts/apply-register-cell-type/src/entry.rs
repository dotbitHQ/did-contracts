use ckb_std::{ckb_constants::Source, high_level};
use core::convert::TryInto;
use core::result::Result;
use das_core::constants::OracleCellType;
use das_core::{
    assert,
    constants::{ScriptType, TypeScript},
    debug,
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_types::{constants::DataType, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running apply-register-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    util::is_system_off(&mut parser)?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"apply_register" {
        debug!("Route to apply_register action ...");

        // Find out ApplyRegisterCells in current transaction.
        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) = util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

        assert!(
            input_cells.len() == 0 && output_cells.len() == 1,
            Error::InvalidTransactionStructure,
            "There should be none ApplyRegisterCell in inputs and one in outputs."
        );

        util::is_cell_use_signall_lock(output_cells[0], Source::Output)?;

        // Verify the outputs_data of ApplyRegisterCell.
        let index = &output_cells[0];
        let data = util::load_cell_data(index.to_owned(), Source::Output)?;

        debug!("Check if the first 32 bytes exists ...");

        assert!(
            data.len() >= 32,
            Error::InvalidCellData,
            "The data of ApplyRegisterCell should start with 32 bytes hash."
        );

        debug!("Check if the height of the ApplyRegisterCell and the HeightCell is consistent ...");

        // Then follows the 8 bytes u64.
        let apply_height = match data.get(32..) {
            Some(bytes) => {
                assert!(
                    bytes.len() == 8,
                    Error::InvalidCellData,
                    "The data of ApplyRegisterCell should end with 8 bytes little-endian uint64."
                );

                u64::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => return Err(Error::InvalidCellData),
        };

        let expected_height = util::load_oracle_data(OracleCellType::Height)?;
        assert!(
            apply_height == expected_height,
            Error::ApplyRegisterCellHeightInvalid,
            "The block number in ApplyRegisterCell data should be the same as which in HeightCell."
        );
    } else if action == b"refund_apply" {
        debug!("Route to refund_apply action ...");

        parser.parse_config(&[DataType::ConfigCellApply])?;
        let config = parser.configs.apply()?;

        // Find out ApplyRegisterCells in current transaction.
        let this_type_script = high_level::load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) = util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

        assert!(
            input_cells.len() == 1 && output_cells.len() == 0,
            Error::InvalidTransactionStructure,
            "There should be one ApplyRegisterCell in inputs and none in outputs."
        );

        debug!("Check if the ApplyRegisterCell is available for refund ...");

        let data = util::load_cell_data(input_cells[0], Source::Input)?;
        // Then follows the 8 bytes u64.
        let apply_height = match data.get(32..) {
            Some(bytes) => {
                assert!(
                    bytes.len() == 8,
                    Error::InvalidCellData,
                    "The data of ApplyRegisterCell should end with 8 bytes little-endian uint64."
                );

                u64::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => return Err(Error::InvalidCellData),
        };
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

        let lock_script = high_level::load_cell_lock(input_cells[0], Source::Input)
            .map_err(|e| Error::from(e))?;
        let transfer_cells =
            util::find_cells_by_script(ScriptType::Lock, lock_script.as_reader(), Source::Output)?;
        assert!(
            transfer_cells.len() == 1,
            Error::InvalidTransactionStructure,
            "There should be one cell in outputs which refund the capacity of the ApplyRegisterCell."
        );

        let expected_capacity = high_level::load_cell_capacity(input_cells[0], Source::Input)
            .map_err(|e| Error::from(e))?;
        let transferred_capacity =
            high_level::load_cell_capacity(transfer_cells[0], Source::Output)
                .map_err(|e| Error::from(e))?;
        assert!(
            transferred_capacity >= expected_capacity - 100_000_000,
            Error::ApplyRegisterRefundCapacityError,
            "The refund of the ApplyRegisterCell should be more than {}, but {} found.",
            expected_capacity - 100_000_000,
            transferred_capacity
        );
    } else if action == b"pre_register" {
        debug!("Route to pre_register action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::PreAccountCellType,
            Source::Output,
            Error::InvalidTransactionStructure,
        )?;
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
