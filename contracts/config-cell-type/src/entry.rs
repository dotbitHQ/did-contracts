use alloc::boxed::Box;
use alloc::string::ToString;
use core::convert::{TryFrom, TryInto};
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::debug;
use ckb_std::high_level::{load_cell_lock_hash, load_cell_type};
use das_core::error::*;
use das_core::{assert, code_to_error, util, warn};
use das_types::constants::{super_lock, Action, DataType};
use das_types::prelude::Entity;
use witness_parser::WitnessesParserV1;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running config-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser.init().map_err(|_err| {
        warn!("{}", _err.to_string());
        code_to_error!(ErrorCode::WitnessDataDecodingError)
    })?;

    // ⚠️ NEVER use util::is_system_off here! That will make it impossible to turn the system back on by updating the ConfigCellMain. ⚠️

    debug!("Route to {:?} action ...", parser.action.to_string());
    if parser.action == Action::Config {
        debug!("Route to config action ...");

        // Finding out ConfigCells in current transaction.
        let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;

        assert!(
            output_cells.len() >= 1,
            ErrorCode::InvalidTransactionStructure,
            "There should be at least one ConfigCell in the outputs."
        );

        debug!("Check if ConfigCells in inputs and outputs are consistent ...");

        if input_cells.len() > 0 {
            assert!(
                input_cells.len() == output_cells.len(),
                ErrorCode::InvalidTransactionStructure,
                "The number of ConfigCell in outputs should be the same as inputs."
            );
        } else {
            // Create new ConfigCells will require super lock to execute after the initialization day of DAS.
            util::require_super_lock()?;
        }

        // Define DAS official super lock.
        let super_lock = super_lock();
        let super_lock_hash = util::blake2b_256(super_lock.as_slice());
        for (i, output_cell_index) in output_cells.into_iter().enumerate() {
            // The ConfigCell in outputs must has the same lock script as super lock.
            // Why we do not limit the input ConfigCell's lock script is because when super lock need to be updated,
            // we need to update this type script at first, then update the ConfigCell after type script deployed.
            let cell_lock_hash =
                load_cell_lock_hash(output_cell_index, Source::Output).map_err(|e| Error::<ErrorCode>::from(e))?;

            assert!(
                cell_lock_hash == super_lock_hash,
                ErrorCode::SuperLockIsRequired,
                "The ConfigCells in outputs must use super lock."
            );

            let output_config_id = get_config_id(output_cell_index, Source::Output)?;

            if input_cells.len() > 0 {
                let input_cell_index = input_cells[i];
                let input_config_id = get_config_id(input_cell_index, Source::Input)?;

                assert!(
                    output_config_id == input_config_id,
                    ErrorCode::InvalidTransactionStructure,
                    "The Config ID in ConfigCells should be the same order in both inputs and outputs."
                );
            }
        }
    } else {
        warn!("The ActionData in witness has an undefined action.");
        return Err(code_to_error!(ErrorCode::ActionNotSupported));
    }

    Ok(())
}

fn get_config_id(cell_index: usize, source: Source) -> Result<DataType, Box<dyn ScriptError>> {
    let cell_type = load_cell_type(cell_index, source)
        .map_err(|e| Error::<ErrorCode>::from(e))?
        .unwrap();
    let args: [u8; 4] = cell_type
        .as_reader()
        .args()
        .raw_data()
        .try_into()
        .map_err(|_| ErrorCode::Encoding)?;
    let config_type = DataType::try_from(u32::from_le_bytes(args)).map_err(|_| ErrorCode::ConfigTypeIsUndefined)?;

    Ok(config_type)
}
