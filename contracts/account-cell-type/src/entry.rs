use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::*,
    debug,
    high_level::{load_cell_lock_hash, load_script},
};
use das_core::{
    constants::{super_lock, ScriptType, ALWAYS_SUCCESS_LOCK},
    error::Error,
    util,
    witness_parser::WitnessesParser,
};
use das_types::constants::ConfigID;

pub fn main() -> Result<(), Error> {
    debug!("====== Running account-cell-type ======");

    // Loading DAS witnesses and parsing the action.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    parser.parse_only_action()?;
    let (action, _) = parser.action();

    debug!("action = {:?}", action);
    if action == b"init_account_chain" {
        debug!("Route to init_account_chain action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        if old_cells.len() != 0 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }
        if new_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

        debug!("Check if super lock has been used in inputs ...");

        let super_lock = super_lock();
        let has_super_lock =
            util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len() > 0;
        if !has_super_lock {
            return Err(Error::SuperLockIsRequired);
        }

        debug!("Check if root AccountCell uses always_success lock ...");

        let index = new_cells[0];
        let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
        let always_success_script_hash = util::blake2b_256(always_success_script.as_slice());
        let lock_script = load_cell_lock_hash(index, Source::Output).map_err(|e| Error::from(e))?;
        if lock_script != always_success_script_hash {
            return Err(Error::WalletRequireAlwaysSuccess);
        }
    } else if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");

        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;

        debug!("The following logic depends on proposal-cell-type.");

        // Find out ProposalCells in current transaction.
        let proposal_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().proposal_cell(),
            Source::Input,
        )?;
        // There must be a ProposalCell consumed in the transaction.
        if proposal_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
        }

    // The AccountCell can be used as long as it is not modified.
    } else {
        debug!("Route to other action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        debug!("Check if AccountCell is consistent.");

        if old_cells.len() != new_cells.len() {
            return Err(Error::CellsMustHaveSameOrderAndNumber);
        }

        for (i, old_index) in old_cells.into_iter().enumerate() {
            let new_index = new_cells[i];
            util::verify_if_cell_capacity_consistent(old_index, new_index)?;
            util::verify_if_cell_consistent(old_index, new_index)?;
        }
    }

    Ok(())
}
