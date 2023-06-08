use alloc::boxed::Box;
use das_core::error::ScriptError;

use crate::{traits::*, create_device_key_list, update_device_key_list, destroy_device_key_list};
pub fn main() -> Result<(), Box<dyn ScriptError>> {
    let mut contract = MyContract::new()?;

    contract.register_action(create_device_key_list::action());
    contract.register_action(update_device_key_list::action());
    contract.register_action(destroy_device_key_list::action());


    contract.run()?;
    Ok(())
}