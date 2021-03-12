use ckb_std::{
    ckb_constants::Source,
    high_level::{load_cell_data, load_script},
};
use core::convert::{TryFrom, TryInto};
use core::result::Result;
use das_core::{constants::ScriptType, debug, error::Error, util, witness_parser::WitnessesParser};
use das_types::constants::ConfigID;
use das_types::{packed::*, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running apply-register-cell-type ======");

    // Loading DAS witnesses and parsing the action.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    parser.parse_only_action()?;
    let (action, _) = parser.action();

    if action == b"apply_register" {
        debug!("Route to apply_register action ...");

        let current = util::load_height()?;

        debug!("Reading ApplyRegisterCell ...");

        // Find out ApplyRegisterCells in current transaction.
        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        // Consuming ApplyRegisterCell is not allowed in apply_register action.
        if old_cells.len() != 0 {
            return Err(Error::ApplyRegisterFoundInvalidTransaction);
        }
        // Only one ApplyRegisterCell can be created at one time.
        if new_cells.len() != 1 {
            return Err(Error::ApplyRegisterFoundInvalidTransaction);
        }

        // Verify the outputs_data of ApplyRegisterCell.
        let index = &new_cells[0];
        let data = load_cell_data(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;

        debug!("Check if first 32 bytes exists ...");

        // The first is a 32 bytes hash.
        match data.get(..32) {
            Some(bytes) => {
                Hash::try_from(bytes).map_err(|_| Error::InvalidCellData)?;
            }
            _ => return Err(Error::InvalidCellData),
        }

        debug!("Check if the ApplyRegisterCell and the HeightCell has the same height...");

        // Then follows the 8 bytes u64.
        let apply_height = match data.get(32..) {
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(Error::InvalidCellData);
                }
                u64::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => return Err(Error::InvalidCellData),
        };

        // The timestamp in ApplyRegisterCell must be the same as the timestamp in TimeCell.
        if apply_height != current {
            return Err(Error::ApplyRegisterCellHeightInvalid);
        }
    } else if action == b"pre_register" {
        debug!("Route to pre_register action ...");

        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;

        debug!(
            "The following logic depends on pre-account-cell-type: {}",
            config.type_id_table().pre_account_cell()
        );

        // Find out PreAccountCells in current transaction.
        let pre_account_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().pre_account_cell(),
            Source::Output,
        )?;
        // There must be a PreAccountCell created in the transaction.
        if pre_account_cells.len() != 1 {
            return Err(Error::ApplyRegisterFoundInvalidTransaction);
        }
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
