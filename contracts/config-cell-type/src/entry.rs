use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_lock_hash, load_cell_type, load_script},
};
use core::convert::{TryFrom, TryInto};
use core::result::Result;
use das_core::util::blake2b_256;
use das_core::{constants::*, error::Error, util, witness_parser::WitnessesParser};
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

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    let (action, _) = parser.parse_only_action()?;

    // Routing by ActionData in witness.
    if action == b"config" {
        debug!("Route to config action ...");

        // Finding out ConfigCells in current transaction.
        let config_cell_type = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &config_cell_type, Source::Output)?;

        // There must be at least one ConfigCell in the outputs.
        if new_cells.len() < 1 {
            return Err(Error::InvalidTransactionStructure);
        }

        debug!("Check super lock of ConfigCells ...");

        let super_lock_hash = blake2b_256(super_lock.as_slice());
        let mut new_config_ids = Vec::new();
        for index in new_cells.clone() {
            // The output ConfigCell must has the same lock script as super lock.
            // Why we do not limit the input ConfigCell's lock script is because when super lock need to be updated,
            // we need to update this type script at first, then update the ConfigCell after type script deployed.
            let cell_lock_hash =
                load_cell_lock_hash(index, Source::Output).map_err(|e| Error::from(e))?;
            if cell_lock_hash != super_lock_hash {
                return Err(Error::CellMustUseSuperLock);
            }

            // Store config IDs for later verification.
            let cell_type = load_cell_type(index, Source::Output)
                .map_err(|e| Error::from(e))?
                .unwrap();
            let args: [u8; 4] = cell_type
                .as_reader()
                .args()
                .raw_data()
                .try_into()
                .map_err(|_| Error::Encoding)?;
            let config_id = ConfigID::try_from(u32::from_le_bytes(args))
                .map_err(|_| Error::ConfigIDIsUndefined)?;
            new_config_ids.push(config_id);
        }

        if old_cells.len() > 0 {
            debug!("Check if ConfigCells in inputs and outputs are consistent ...");

            // If ConfigCells exist in inputs, it means updating and the number of ConfigCell in inputs and outputs must be the same.
            if old_cells.len() != new_cells.len() {
                return Err(Error::InvalidTransactionStructure);
            }

            let mut old_config_ids = Vec::new();
            for index in new_cells {
                // Store config IDs for later verification.
                let cell_type = load_cell_type(index, Source::Output)
                    .map_err(|e| Error::from(e))?
                    .unwrap();
                let args: [u8; 4] = cell_type
                    .as_reader()
                    .args()
                    .raw_data()
                    .try_into()
                    .map_err(|_| Error::Encoding)?;
                let config_id = ConfigID::try_from(u32::from_le_bytes(args))
                    .map_err(|_| Error::ConfigIDIsUndefined)?;
                old_config_ids.push(config_id);
            }

            // Config IDs in inputs and outputs must be the consistent.
            if old_config_ids != new_config_ids {
                return Err(Error::InvalidTransactionStructure);
            }
        }
    } else {
        return Err(Error::ActionNotSupported);
    }

    Ok(())
}
