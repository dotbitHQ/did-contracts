use alloc::vec;
use ckb_std::{ckb_constants::Source, high_level::load_script};
use das_core::{
    assert,
    constants::{ScriptType, TypeScript},
    debug,
    error::Error,
    util,
};
use das_types::constants::DataType;

pub fn main() -> Result<(), Error> {
    debug!("====== Running ref-cell-type ======");

    let action_data = util::load_das_action()?;
    let action = action_data.as_reader().action().raw_data();
    if action == b"confirm_proposal" {
        debug!("Route to confirm_proposal action ...");
        let mut parser = util::load_das_witnesses(Some(vec![DataType::ConfigCellMain]))?;
        util::require_type_script(
            &mut parser,
            TypeScript::ProposalCellType,
            Source::Input,
            Error::ProposalFoundInvalidTransaction,
        )?;
    } else if action == b"transfer_account" || action == b"edit_manager" {
        debug!("Route to transfer_account/edit_manager action ...");
        let mut parser = util::load_das_witnesses(Some(vec![DataType::ConfigCellMain]))?;
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
        let (input_cells, output_cells) =
            util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, &this_type_script)?;

        assert!(
            input_cells.len() == output_cells.len(),
            Error::CellsMustHaveSameOrderAndNumber,
            "The RefCells in inputs should have the same number and order as those in outputs."
        );

        util::is_inputs_and_outputs_consistent(input_cells, output_cells)?;
    }

    Ok(())
}
