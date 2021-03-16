use ckb_std::{ckb_constants::Source, debug, high_level::load_script};
use das_core::{
    constants::{ScriptType, TypeScript},
    error::Error,
    util,
    witness_parser::WitnessesParser,
};

pub fn main() -> Result<(), Error> {
    debug!("====== Running ref-cell-type ======");

    // Loading and parsing DAS witnesses.
    let witnesses = util::load_das_witnesses()?;
    let mut parser = WitnessesParser::new(witnesses)?;
    parser.parse_only_action()?;
    let (action, _) = parser.action();

    if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"transfer_account" || action == b"edit_manager" {
        debug!("Route to transfer_account/edit_manager action ...");
        util::require_type_script(
            &mut parser,
            TypeScript::AccountCellType,
            Source::Input,
            Error::AccountCellFoundInvalidTransaction,
        )?;
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
