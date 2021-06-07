use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, load_script},
};
use core::convert::TryInto;
use core::result::Result;
use das_core::{
    assert,
    constants::{ScriptType, TypeScript},
    debug,
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_types::prelude::*;

pub fn main() -> Result<(), Error> {
    debug!("====== Running apply-register-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    util::is_system_off(&mut parser)?;

    let action_data = parser.parse_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"apply_register" {
        debug!("Route to apply_register action ...");

        debug!("Reading ApplyRegisterCell ...");

        // Find out ApplyRegisterCells in current transaction.
        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let (input_cells, output_cells) = util::find_cells_by_script_in_inputs_and_outputs(
            ScriptType::Type,
            this_type_script.as_reader(),
        )?;

        assert!(
            input_cells.len() == 0 && output_cells.len() == 1,
            Error::ApplyRegisterFoundInvalidTransaction,
            "There should be none ApplyRegisterCell in inputs and one in outputs."
        );

        util::is_cell_use_signall_lock(output_cells[0], Source::Output)?;

        // Verify the outputs_data of ApplyRegisterCell.
        let index = &output_cells[0];
        let data = load_cell_data(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;

        debug!("Check if first 32 bytes exists ...");

        assert!(
            data.get(..32).is_some(),
            Error::InvalidCellData,
            "The data of ApplyRegisterCell should start with 32 bytes hash."
        );

        debug!("Check if the ApplyRegisterCell and the HeightCell has the same height...");

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

        let expected_height = util::load_height()?;
        assert!(
            apply_height == expected_height,
            Error::ApplyRegisterCellHeightInvalid,
            "The block number in ApplyRegisterCell data should be the same as which in HeightCell."
        );
    } else if action == b"pre_register" {
        debug!("Route to pre_register action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::PreAccountCellType,
            Source::Output,
            Error::PreRegisterFoundInvalidTransaction,
        )?;
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
