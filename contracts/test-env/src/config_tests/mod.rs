use alloc::boxed::Box;
use core::result::Result;

use das_core::error::ScriptError;
use das_core::config::Config;

pub fn test_config_account_loading() -> Result<(), Box<dyn ScriptError>> {
    let _config_main = Config::get_instance().account()?;

    Ok(())
}

pub fn test_config_records_key_namespace_loading() -> Result<(), Box<dyn ScriptError>> {
    let _config_namespace = Config::get_instance().record_key_namespace()?;

    Ok(())
}
