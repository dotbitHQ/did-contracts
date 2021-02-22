use ckb_std::high_level::{load_cell_capacity, load_cell_lock, load_cell_type};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_lock_hash, load_script},
};
use das_core::{
    constants::{super_lock, ScriptType, ALWAYS_SUCCESS_LOCK, WALLET_CELL_BASIC_CAPACITY},
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_types::{packed::AccountCellData, prelude::Entity};

pub fn main() -> Result<(), Error> {
    debug!("====== Running wallet-cell-type ======");

    debug!("Find out WalletCells ...");

    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let old_cells = util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let new_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    match util::load_das_witnesses() {
        Ok(witnesses) => {
            let action = WitnessesParser::parse_only_action(&witnesses)?;
            if action.as_reader().raw_data() == "create_wallet".as_bytes() {
                debug!("Route to create_wallet action ...");

                let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
                let always_success_script_hash =
                    util::blake2b_256(always_success_script.as_slice());

                debug!("Check if super lock has been used in inputs ...");

                let super_lock = super_lock();
                let has_super_lock =
                    util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len()
                        > 0;
                if !has_super_lock {
                    return Err(Error::SuperLockIsRequired);
                }

                debug!("Check if any WalletCell be consumed ...");

                if old_cells.len() != 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }

                debug!("Check if all WalletCells use always_success lock ...");
                debug!("Check if all WalletCells has correct basic capacity ...");

                for i in new_cells {
                    let lock_script =
                        load_cell_lock_hash(i, Source::Output).map_err(|e| Error::from(e))?;
                    if lock_script != always_success_script_hash {
                        return Err(Error::WalletRequireAlwaysSuccess);
                    }

                    let capacity =
                        load_cell_capacity(i, Source::Output).map_err(|e| Error::from(e))?;
                    if capacity > WALLET_CELL_BASIC_CAPACITY {
                        return Err(Error::WalletBaseCapacityIsWrong);
                    }
                }
            } else if action.as_reader().raw_data() == "recycle_wallet".as_bytes() {
                debug!("Route to recycle_wallet action ...");

                debug!("Check if super lock has been used in inputs ...");

                // TODO Hardcode a lock to recycle WalletCells.
                let super_lock = super_lock();
                let has_super_lock =
                    util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len()
                        > 0;
                if !has_super_lock {
                    return Err(Error::SuperLockIsRequired);
                }

                debug!("Check if any WalletCell still exists ...");

                if old_cells.len() == 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
                if new_cells.len() != 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
            } else if action.as_reader().raw_data() == "withdraw_from_wallet".as_bytes() {
                debug!("Route to recycle_wallet action ...");

                debug!("For WalletCell, check if only capacity is reduced ...");

                // Only one WalletCell can be withdrawn at a time.
                if old_cells.len() != 1 || new_cells.len() != 1 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
                let old_index = old_cells[0];
                let new_index = new_cells[0];
                util::verify_if_cell_consistent(old_index, new_index)?;
                util::verify_if_cell_capacity_reduced(old_index, new_index)?;

                let wallet_type = load_cell_type(old_index, Source::Input)
                    .map_err(|err| Error::from(err))?
                    .unwrap();
                let id_in_wallet = wallet_type.as_reader().args().raw_data();

                debug!("Check if OwnerCell and AccountCell exists ...");

                // Load config from witnesses.
                let parser = WitnessesParser::new(witnesses)?;
                let config = util::load_config(&parser)?;

                // Find out RefCells in current transaction.
                let old_ref_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.as_reader().type_id_table().ref_cell(),
                    Source::Input,
                )?;
                let new_ref_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.as_reader().type_id_table().ref_cell(),
                    Source::Output,
                )?;
                util::verify_if_cell_consistent(old_ref_index, new_ref_index)?;
                util::verify_if_cell_capacity_consistent(old_ref_index, new_ref_index)?;

                // Find out AccountCells in current transaction.
                let old_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.as_reader().type_id_table().account_cell(),
                    Source::Input,
                )?;
                let new_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.as_reader().type_id_table().account_cell(),
                    Source::Output,
                )?;
                util::verify_if_cell_consistent(old_account_index, new_account_index)?;
                util::verify_if_cell_capacity_consistent(old_account_index, new_account_index)?;

                debug!("Check if OwnerCell has permission to withdraw from WalletCell ...");
                // User must have the owner permission to withdraw CKB from the WalletCell.

                let ref_type = load_cell_type(old_ref_index, Source::Input)
                    .map_err(|err| Error::from(err))?
                    .unwrap();
                let id_in_ref = ref_type.as_reader().args().raw_data();
                let (_, _, entity) =
                    util::get_cell_witness(&parser, old_account_index, Source::Input)?;
                let account_cell_witness =
                    AccountCellData::from_slice(entity.as_reader().raw_data())
                        .map_err(|_| Error::WitnessEntityDecodingError)?;

                debug!("  Check if RefCell is related to WalletCell ...");

                if id_in_wallet != id_in_ref {
                    return Err(Error::WalletPermissionInvalid);
                }

                debug!("  Check if RefCell is related to AccountCell ...");

                let expected_id = account_cell_witness.as_reader().id().raw_data();
                if expected_id != id_in_ref {
                    return Err(Error::WalletPermissionInvalid);
                }

                debug!("  Check if RefCell has owner permission ...");

                let lock =
                    load_cell_lock(old_ref_index, Source::Input).map_err(|err| Error::from(err))?;
                let expected_lock = account_cell_witness.owner_lock();
                if expected_lock.as_slice() != lock.as_slice() {
                    return Err(Error::WalletPermissionInvalid);
                }
            } else {
                return Err(Error::ActionNotSupported);
            }

            Ok(())
        }
        _ => {
            debug!("Route to non-action ...");

            debug!("Check if WalletCell is consistent and has more capacity than before.");

            if old_cells.len() != new_cells.len() {
                return Err(Error::CellsMustHaveSameOrderAndNumber);
            }

            for (i, old_index) in old_cells.into_iter().enumerate() {
                let new_index = new_cells[i];
                util::verify_if_cell_capacity_increased(old_index, new_index)?;
                util::verify_if_cell_consistent(old_index, new_index)?;
            }

            Ok(())
        }
    }
}
