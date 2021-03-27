use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_lock_hash, load_cell_type, load_script},
};
use core::convert::{TryFrom, TryInto};
use core::result::Result;
use das_core::{assert, constants::*, error::Error, util, warn};
use das_types::{constants::ConfigID, prelude::Entity};

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

    let action_data = util::load_das_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"config" {
        debug!("Route to config action ...");

        // Finding out ConfigCells in current transaction.
        let config_cell_type = load_script().map_err(|e| Error::from(e))?;
        let input_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Input)?;
        let output_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Output)?;

        assert!(
            output_cells.len() >= 1,
            Error::InvalidTransactionStructure,
            "There should be at least one ConfigCell in the outputs."
        );

        debug!("Check if ConfigCells in inputs and outputs are consistent ...");

        if input_cells.len() > 0 {
            assert!(
                input_cells.len() == output_cells.len(),
                Error::InvalidTransactionStructure,
                "The number of ConfigCell in outputs should be the same as inputs."
            );
        }

        let super_lock_hash = util::blake2b_256(super_lock.as_slice());
        for (i, output_cell_index) in output_cells.into_iter().enumerate() {
            // The ConfigCell in outputs must has the same lock script as super lock.
            // Why we do not limit the input ConfigCell's lock script is because when super lock need to be updated,
            // we need to update this type script at first, then update the ConfigCell after type script deployed.
            let cell_lock_hash = load_cell_lock_hash(output_cell_index, Source::Output)
                .map_err(|e| Error::from(e))?;

            assert!(
                cell_lock_hash == super_lock_hash,
                Error::CellMustUseSuperLock,
                "The ConfigCells in outputs must use super lock."
            );

            let output_config_id = get_config_id(output_cell_index, Source::Output)?;

            if input_cells.len() > 0 {
                let input_cell_index = input_cells[i];
                let input_config_id = get_config_id(input_cell_index, Source::Input)?;

                assert!(
                    output_config_id == input_config_id,
                    Error::InvalidTransactionStructure,
                    "The Config ID in ConfigCells should be the same order in both inputs and outputs."
                );
            }
        }
    } else {
        warn!("The ActionData in witness has an undefined action.");
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}

fn get_config_id(cell_index: usize, source: Source) -> Result<ConfigID, Error> {
    let cell_type = load_cell_type(cell_index, source)
        .map_err(|e| Error::from(e))?
        .unwrap();
    let args: [u8; 4] = cell_type
        .as_reader()
        .args()
        .raw_data()
        .try_into()
        .map_err(|_| Error::Encoding)?;
    let config_id =
        ConfigID::try_from(u32::from_le_bytes(args)).map_err(|_| Error::ConfigIDIsUndefined)?;

    Ok(config_id)
}
