use alloc::borrow::ToOwned;
use ckb_std::{
    ckb_constants::Source,
    debug,
    // debug,
    high_level::{load_cell_lock, load_script},
};
use core::result::Result;
use das_core::{constants::*, error::Error, util, witness_parser::WitnessesParser};

pub fn main() -> Result<(), Error> {
    debug!("====== Running config-cell-type ======");

    // Define DAS official super lock.
    let super_lock = super_lock();

    debug!("Check if super lock has been used in inputs ...");

    // Limit this type script must be used with super lock.
    let has_super_lock =
        util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len() > 0;
    if !has_super_lock {
        return Err(Error::SuperLockIsRequired);
    }

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let parser = WitnessesParser::new(witnesses)?;

    // Routing by ActionData in witness.
    let action = parser.action.as_reader().raw_data();
    if action == "config".as_bytes() {
        debug!("Route to config action ...");

        // Finding out ConfigCells in current transaction.
        let config_cell_type = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Output)?;

        // There must be one ConfigCell in the outputs, no more and no less.
        if new_cells.len() != 1 {
            return Err(Error::InvalidTransactionStructure);
        }
        let index = &new_cells[0];
        // The ConfigCell must be the first one in the outputs.
        if index.to_owned() != 0 {
            return Err(Error::InvalidTransactionStructure);
        }

        debug!("Check if the witness of config cell is correct ...");

        util::verify_cells_witness(&parser, index.to_owned(), Source::Output)?;

        // The output ConfigCell must has the same lock script as super lock.
        // Why we do not limit the input ConfigCell's lock script is because when super lock need to be updated,
        // we need to update this type script at first, then update the ConfigCell after type script deployed.
        let cell_lock =
            load_cell_lock(index.to_owned(), Source::Output).map_err(|e| Error::from(e))?;
        if !util::is_entity_eq(&cell_lock, &super_lock) {
            return Err(Error::CellMustUseSuperLock);
        }

        if old_cells.len() > 0 {
            // Only one ConfigCell is allowed in inputs at most.
            if old_cells.len() != 1 {
                return Err(Error::InvalidTransactionStructure);
            }
            let index = &old_cells[0];
            // The ConfigCell must be the first one in the inputs.
            if index.to_owned() != 0 {
                return Err(Error::InvalidTransactionStructure);
            }

            util::verify_cells_witness(&parser, index.to_owned(), Source::Input)?;
        }
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
