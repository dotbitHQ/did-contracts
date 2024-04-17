use alloc::boxed::Box;
use core::result::Result;

use das_core::config::Config;
use das_core::error::{ErrorCode, ScriptError};
use das_core::{code_to_error, das_assert};

pub fn test_config_account_loading() -> Result<(), Box<dyn ScriptError>> {
    let config_account = Config::get_instance().account()?;

    let expected_basic_capacity = 20_600_000_000;
    das_assert!(
        u64::from(config_account.basic_capacity()) == expected_basic_capacity,
        ErrorCode::UnittestError,
        "The basic_capacity should be {}",
        expected_basic_capacity
    );

    Ok(())
}

pub fn test_config_records_key_namespace_loading() -> Result<(), Box<dyn ScriptError>> {
    let config_namespace = Config::get_instance().record_key_namespace()?;

    das_assert!(
        !config_namespace.is_empty(),
        ErrorCode::UnittestError,
        "The record_key_namespace should not be empty"
    );

    Ok(())
}
