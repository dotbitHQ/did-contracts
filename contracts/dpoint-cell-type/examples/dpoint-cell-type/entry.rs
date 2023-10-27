use alloc::boxed::Box;

// use das_core::code_to_error;
use das_core::constants::TypeScript;
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
// use das_core::witness_parser::general_witness_parser::{get_witness_parser, FromWitness, Witness};
// use das_types::packed::ActionData;
// use dpoint_cell_type::error::ErrorCode;
use das_core::debug;
use das_core::util;


pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running dpoint-cell-type ======");

    let parser = WitnessesParser::new()?;
    util::exec_by_type_id(&parser, TypeScript::EIP712Lib, &[])?;

    Ok(())
}
