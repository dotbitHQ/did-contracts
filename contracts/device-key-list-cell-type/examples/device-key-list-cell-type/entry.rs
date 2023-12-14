use alloc::boxed::Box;

use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParserLegacy;
use das_core::{code_to_error, util};
use das_types::constants::{WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use das_types::packed::ActionData;
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::traits::*;
use crate::{create_device_key_list, destroy_device_key_list, update_device_key_list};
pub fn main() -> Result<(), Box<dyn ScriptError>> {
    let mut parser = WitnessesParserLegacy::new()?;
    parser.parse_cell()?;
    let witness = util::load_das_witnesses(parser.witnesses[0].0)?;
    let action_data = ActionData::from_slice(witness.get(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..).unwrap())
        .map_err(|_| code_to_error!(ErrorCode::VerificationError))?;

    let mut actions = RegisteredActions::default();
    actions.register_action(create_device_key_list::action());
    actions.register_action(update_device_key_list::action());
    actions.register_action(destroy_device_key_list::action());

    let active_action = actions
        .get_active_action(&action_data)
        .ok_or(code_to_error!(ErrorCode::ActionNotSupported))?;

    let mut contract = MyContract::new(parser, action_data)?;

    contract.run_against_action(&active_action)?;
    Ok(())
}
