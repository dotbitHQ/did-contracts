use ckb_std::{ckb_constants::Source, debug, high_level::load_script};
use das_core::{
    constants::{super_lock, ScriptType},
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

        let super_lock = super_lock();

        // Limit this type script must be used with super lock.
        let has_super_lock =
            util::find_cells_by_script(ScriptType::Lock, &super_lock, Source::Input)?.len() > 0;
        if !has_super_lock {
            return Err(Error::SuperLockIsRequired);
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
