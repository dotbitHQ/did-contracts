use alloc::boxed::Box;

use das_core::code_to_error;
use das_core::contract::defult_structs::{RegisteredActions, MyContract};
use das_core::contract::traits::FSMContract;
use das_core::error::ScriptError;
use das_core::witness_parser::general_witness_parser::{get_witness_parser, FromWitness, Witness};
use das_types::packed::ActionData;
use device_key_list_cell_type::error::ErrorCode;

use crate::{create_device_key_list, destroy_device_key_list, update_device_key_list};
pub fn main() -> Result<(), Box<dyn ScriptError>> {
    // let mut parser = WitnessesParser::new()?;
    // parser.parse_cell()?;
    let witness = get_witness_parser().get_das_witness(0)?;
    let action_data = ActionData::from_witness(&Witness::Loaded(witness.clone()))?;
    // let action_data = ActionData::from_slice(witness.get(WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..).unwrap())
    //     .map_err(|_| code_to_error!(ErrorCode::VerificationError))?;

    let mut actions = RegisteredActions::default();
    actions.register_action(create_device_key_list::action());
    actions.register_action(update_device_key_list::action());
    actions.register_action(destroy_device_key_list::action());

    let active_action = actions
        .get_active_action(&action_data)
        .ok_or(code_to_error!(ErrorCode::ActionNotSupported))?;

    let mut contract = MyContract::new(action_data)?;

    contract.run_against_action(&active_action)?;
    Ok(())
}
