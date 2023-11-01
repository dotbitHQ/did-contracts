use alloc::boxed::Box;

use das_core::contract::defult_structs::*;
use das_core::contract::traits::FSMContract;
use das_core::error::{ErrorCode, ScriptError};
use das_core::witness_parser::general_witness_parser::*;
use das_core::{code_to_error, debug};
use das_types::packed::*;

use super::{burn_dp, mint_dp, transfer_dp};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running dpoint-cell-type ======");

    let witness = get_witness_parser().get_das_witness(0)?;
    let action_data = ActionData::from_witness(&Witness::Loaded(witness.clone()))?;

    let mut actions = RegisteredActions::default();
    actions.register_action(mint_dp::action()?);
    actions.register_action(transfer_dp::action()?);
    actions.register_action(burn_dp::action()?);

    let active_action = actions
        .get_active_action(&action_data)
        .ok_or(code_to_error!(ErrorCode::ActionNotSupported))?;

    let mut contract = MyContract::new(action_data)?;

    contract.run_against_action(&active_action)?;

    Ok(())
}
