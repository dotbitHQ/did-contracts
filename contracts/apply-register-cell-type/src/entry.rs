use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_data, load_script},
};
use core::convert::{TryFrom, TryInto};
use core::result::Result;
use das_core::{constants::ScriptType, error::Error, util, witness_parser::WitnessesParser};
use das_types::{packed::*, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running apply-register-cell-type ======");

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let parser = WitnessesParser::new(witnesses)?;

    // Define nervos official TimeCell type script.
    let current = util::load_timestamp()?;
    let config = util::load_config(&parser)?;

    debug!("Reading ApplyRegisterCell ...");

    // Find out ApplyRegisterCells in current transaction.
    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let old_cells = util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let new_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    // Routing by ActionData in witness.
    let action = parser.action.as_reader().raw_data();
    if action == "apply_register".as_bytes() {
        debug!("Route to apply_register action ...");

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

        // The first is a 32 bytes hash.
        match data.get(..32) {
            Some(bytes) => {
                Hash::try_from(bytes).map_err(|_| Error::InvalidCellData)?;
            }
            _ => return Err(Error::InvalidCellData),
        }

        // Then follows the 8 bytes u64.
        let apply_timestamp = match data.get(32..) {
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(Error::InvalidCellData);
                }
                u64::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => return Err(Error::InvalidCellData),
        };

        debug!("Timestamp: {:02x?}", apply_timestamp);

        // The timestamp in ApplyRegisterCell must be the same as the timestamp in TimeCell.
        if apply_timestamp != current {
            return Err(Error::ApplyRegisterCellTimeError);
        }
    } else if action == "pre_register".as_bytes() {
        debug!("Route to pre_register action ...");

        debug!(
            "depends on pre-account-cell-type: {}",
            config.as_reader().type_id_table().pre_account_cell()
        );

        // Only one ApplyRegisterCell can be consumed at one time.
        if old_cells.len() != 1 {
            return Err(Error::ApplyRegisterFoundInvalidTransaction);
        }
        // Creating ApplyRegisterCell is not allowed in apply_register action.
        if new_cells.len() != 0 {
            return Err(Error::ApplyRegisterFoundInvalidTransaction);
        }

        debug!("Check existence of the PreAccountCell ...");

        // Find out PreAccountCells in current transaction.
        let pre_account_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.as_reader().type_id_table().pre_account_cell(),
            Source::Output,
        )?;
        // There must be a PreAccountCell created in the transaction.
        if cfg!(not(debug_assertions)) {
            if pre_account_cells.len() != 1 {
                return Err(Error::ApplyRegisterFoundInvalidTransaction);
            }
        }

        // Read the apply timestamp from outputs_data of ApplyRegisterCell.
        let index = &old_cells[0];
        let data = load_cell_data(index.to_owned(), Source::Input).map_err(|e| Error::from(e))?;
        let apply_timestamp = match data.get(32..) {
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(Error::InvalidCellData);
                }
                u64::from_le_bytes(bytes.try_into().unwrap())
            }
            _ => return Err(Error::InvalidCellData),
        };

        // Check that the ApplyRegisterCell has existed long enough, but has not yet timed out.
        let apply_min_waiting_time =
            u32::from(config.as_reader().apply_min_waiting_time().to_entity());
        let apply_max_waiting_time =
            u32::from(config.as_reader().apply_max_waiting_time().to_entity());
        let passed_time = current - apply_timestamp;

        debug!(
            "Has passed {}s after apply.(min waiting: {}s, max waiting: {}s)",
            passed_time, apply_min_waiting_time, apply_max_waiting_time
        );

        if passed_time < apply_min_waiting_time as u64 {
            return Err(Error::ApplyRegisterNeedWaitLonger);
        }
        if passed_time > apply_max_waiting_time as u64 {
            return Err(Error::ApplyRegisterHasTimeout);
        }
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
