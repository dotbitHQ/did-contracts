use ckb_std::{ckb_constants::Source, debug, high_level::load_script};
use das_core::{constants::ScriptType, error::Error, util, witness_parser::WitnessesParser};
use das_types::constants::ConfigID;

pub fn main() -> Result<(), Error> {
    debug!("====== Running ref-cell-type ======");

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    parser.parse_only_action()?;
    let (action, _) = parser.action();

    if action == b"confirm_proposal" {
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
    } else if action == b"transfer_account" || action == b"edit_manager" {
        debug!("Route to transfer_account/edit_manager action ...");

        parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
        let config = parser.configs().main()?;

        debug!("The following logic depends on account-cell-type.");

        // Find out AccountCells in current transaction.
        let account_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.type_id_table().account_cell(),
            Source::Input,
        )?;
        // There must be a AccountCell consumed in the transaction.
        if account_cells.len() != 1 {
            return Err(Error::AccountCellFoundInvalidTransaction);
        }

    // The RefCell can be used as long as it is not modified.
    } else {
        debug!("Route to other action ...");

        let this_type_script = load_script().map_err(|e| Error::from(e))?;
        let old_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
        let new_cells =
            util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

        debug!("Check if RefCell is consistent.");

        if old_cells.len() != new_cells.len() {
            return Err(Error::CellsMustHaveSameOrderAndNumber);
        }

        for (i, old_index) in old_cells.into_iter().enumerate() {
            let new_index = new_cells[i];
            util::is_cell_capacity_equal((old_index, Source::Input), (new_index, Source::Output))?;
            util::is_cell_consistent((old_index, Source::Input), (new_index, Source::Output))?;
        }
    }

    Ok(())
}
