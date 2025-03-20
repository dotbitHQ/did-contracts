use alloc::boxed::Box;
use core::result::Result;

use das_core::error::{ScriptError, *};
use das_core::{assert, code_to_error};
use witness_parser::WitnessesParserV1;

pub fn test_witness_parser_action_params_data_init() -> Result<(), Box<dyn ScriptError>> {
    let parser = WitnessesParserV1::get_instance();

    assert!(
        1 == parser.action_params_data.len(),
        ErrorCode::UnittestError,
        "ActionParamsData length should be 1 for unit test."
    );

    let expected_name = b"test_witness_parser_action_params_data_init";

    assert!(
        expected_name.to_vec() == parser.action_params_data[0],
        ErrorCode::UnittestError,
        "The name in witness unit test should be equal."
    );

    Ok(())
}
