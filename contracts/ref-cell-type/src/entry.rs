use ckb_std::{ckb_constants::Source, debug, high_level::load_script};
use das_core::{constants::ScriptType, error::Error, util, witness_parser::WitnessesParser};

pub fn main() -> Result<(), Error> {
    debug!("====== Running ref-cell-type ======");

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let action_data = WitnessesParser::parse_only_action(&witnesses)?;
    let action = action_data.as_reader().action().raw_data();

    if action == "confirm_proposal".as_bytes() {
        debug!("Route to confirm_proposal action ...");

        let config = WitnessesParser::parse_only_config(&witnesses)?;

        debug!("The following logic depends on proposal-cell-type.");

        // Find out ProposalCells in current transaction.
        let proposal_cells = util::find_cells_by_type_id(
            ScriptType::Type,
            config.as_reader().type_id_table().proposal_cell(),
            Source::Input,
        )?;
        // There must be a ProposalCell consumed in the transaction.
        if proposal_cells.len() != 1 {
            return Err(Error::ProposalFoundInvalidTransaction);
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
            util::verify_if_cell_capacity_consistent(old_index, new_index)?;
            util::verify_if_cell_consistent(old_index, new_index)?;
        }
    }

    Ok(())
}
