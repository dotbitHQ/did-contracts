use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_lock, load_cell_lock_hash, load_cell_type, load_script},
};
use das_core::{
    constants::{wallet_maker_lock, ScriptType, TypeScript, ALWAYS_SUCCESS_LOCK},
    data_parser::ref_cell,
    error::Error,
    util,
};
use das_types::{
    constants::{ConfigID, DataType},
    packed::AccountCellData,
    prelude::Entity,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running wallet-cell-type ======");

    debug!("Find out WalletCells ...");

    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let old_cells = util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let new_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    match util::load_das_action() {
        Ok(action_data) => {
            let action = action_data.as_reader().action().raw_data();
            if action == b"create_wallet" {
                debug!("Route to create_wallet action ...");

                let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
                let always_success_script_hash =
                    util::blake2b_256(always_success_script.as_slice());

                debug!("Check if wallet maker lock has been used in inputs ...");

                let expected_lock = wallet_maker_lock();
                let has_expected_lock =
                    util::find_cells_by_script(ScriptType::Lock, &expected_lock, Source::Input)?
                        .len()
                        > 0;
                if !has_expected_lock {
                    return Err(Error::WalletRequireWalletMakerLock);
                }

                debug!("Check if any WalletCell be consumed ...");

                if old_cells.len() != 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }

                debug!("Check if all WalletCells use always_success lock ...");

                for i in new_cells {
                    let lock_script_hash =
                        load_cell_lock_hash(i, Source::Output).map_err(|e| Error::from(e))?;
                    if lock_script_hash != always_success_script_hash {
                        let lock_script =
                            load_cell_lock(i, Source::Output).map_err(|e| Error::from(e))?;
                        debug!(
                            "The lock script of WalletCell(outputs[{}]) is invalid: {:?} != {:?} => true",
                            i, always_success_script, lock_script
                        );
                        return Err(Error::WalletRequireAlwaysSuccess);
                    }
                }
            } else if action == b"recycle_wallet" {
                debug!("Route to recycle_wallet action ...");

                debug!("Check if wallet maker lock has been used in inputs ...");

                let expected_lock = wallet_maker_lock();
                let has_expected_lock =
                    util::find_cells_by_script(ScriptType::Lock, &expected_lock, Source::Input)?
                        .len()
                        > 0;
                if !has_expected_lock {
                    return Err(Error::WalletRequireWalletMakerLock);
                }

                debug!("Check if any WalletCell still exists ...");

                if old_cells.len() == 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
                if new_cells.len() != 0 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
            } else if action == b"withdraw_from_wallet" {
                debug!("Route to withdraw_from_wallet action ...");

                let mut parser = util::load_das_witnesses(None)?;
                parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
                parser.parse_all_data()?;
                let config = parser.configs().main()?;

                debug!("For WalletCell, check if only capacity is reduced ...");

                // Only one WalletCell can be withdrawn at a time.
                if old_cells.len() != 1 || new_cells.len() != 1 {
                    return Err(Error::WalletFoundInvalidTransaction);
                }
                let old_index = old_cells[0];
                let new_index = new_cells[0];
                util::is_cell_consistent((old_index, Source::Input), (new_index, Source::Output))?;
                util::is_cell_capacity_lte(
                    (old_index, Source::Input),
                    (new_index, Source::Output),
                )?;

                let wallet_cell_data = load_cell_type(old_index, Source::Input)
                    .map_err(|e| Error::from(e))?
                    .unwrap();
                let id_in_wallet = wallet_cell_data.as_reader().args().raw_data();
                debug!("{:?}", id_in_wallet);

                debug!("Check if OwnerCell and AccountCell exists ...");

                // Find out RefCells in current transaction.
                let old_ref_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().ref_cell(),
                    Source::Input,
                )?;
                let new_ref_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().ref_cell(),
                    Source::Output,
                )?;
                util::is_cell_consistent(
                    (old_ref_index, Source::Input),
                    (new_ref_index, Source::Output),
                )?;
                util::is_cell_capacity_equal(
                    (old_ref_index, Source::Input),
                    (new_ref_index, Source::Output),
                )?;

                // Find out AccountCells in current transaction.
                let old_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().account_cell(),
                    Source::Input,
                )?;
                let new_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().account_cell(),
                    Source::Output,
                )?;
                util::is_cell_consistent(
                    (old_account_index, Source::Input),
                    (new_account_index, Source::Output),
                )?;
                util::is_cell_capacity_equal(
                    (old_account_index, Source::Input),
                    (new_account_index, Source::Output),
                )?;

                debug!("Check if OwnerCell has permission to withdraw from WalletCell ...");
                // User must have the owner permission to withdraw CKB from the WalletCell.

                let ref_data = util::load_cell_data(old_ref_index, Source::Input)?;
                let id_in_ref = ref_cell::get_id(&ref_data);
                let (_, _, entity) = parser.verify_and_get(old_account_index, Source::Input)?;
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
            } else if action == b"recycle_expired_account_by_keeper" {
                debug!("Route to recycle_expired_account_by_keeper action ...");
                let mut parser = util::load_das_witnesses(Some(vec![DataType::ConfigCellMain]))?;
                util::require_type_script(
                    &mut parser,
                    TypeScript::AccountCellType,
                    Source::Input,
                    Error::AccountCellFoundInvalidTransaction,
                )?;
            } else {
                debug!("Route to other action ...");

                verify_if_only_capacity_increased(old_cells, new_cells)?;
            }
        }
        // WalletCell can be also used in any transactions without the ActionData of DAS.
        Err(Error::WitnessActionNotFound) => {
            debug!("Route to non-action ...");

            verify_if_only_capacity_increased(old_cells, new_cells)?;
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn verify_if_only_capacity_increased(
    old_cells: Vec<usize>,
    new_cells: Vec<usize>,
) -> Result<(), Error> {
    debug!("Check if WalletCell is consistent and has more capacity than before.");

    if old_cells.len() != new_cells.len() {
        return Err(Error::CellsMustHaveSameOrderAndNumber);
    }

    for (i, old_index) in old_cells.into_iter().enumerate() {
        let new_index = new_cells[i];
        util::is_cell_capacity_gte((old_index, Source::Input), (new_index, Source::Output))?;
        util::is_cell_consistent((old_index, Source::Input), (new_index, Source::Output))?;
    }

    Ok(())
}
