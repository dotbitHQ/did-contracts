use alloc::boxed::Box;

use das_core::contract::defult_structs::*;
use das_core::contract::traits::FSMContract;
use das_core::error::{ErrorCode, ScriptError};
use das_core::{code_to_error, debug, util};
use witness_parser::WitnessesParserV1;

use super::{burn_dp, mint_dp, transfer_dp};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running dpoint-cell-type ======");

    let parser = WitnessesParserV1::get_instance();
    parser
        .init()
        .map_err(|_err| code_to_error!(ErrorCode::WitnessDataDecodingError))?;

    util::is_system_off()?;

    let mut actions = RegisteredActions::default();
    actions.register_action(mint_dp::action()?);
    actions.register_action(transfer_dp::action()?);
    actions.register_action(burn_dp::action()?);

    let action_data = parser.get_action_data().clone();
    let active_action = actions
        .get_active_action(&action_data)
        .ok_or(code_to_error!(ErrorCode::ActionNotSupported))?;

    let mut contract = MyContract::new(action_data)?;

    contract.run_against_action(&active_action)?;

    Ok(())
}
