use alloc::boxed::Box;
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::constants::{das_lock, TypeScript};
use das_core::error::*;
use das_core::witness_parser::WitnessesParser;
use das_core::{assert, assert_lock_equal, code_to_error, debug, util, verifiers};

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running reverse-record-root-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    util::is_system_off(&parser)?;

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| ErrorCode::ActionNotSupported)?
    );

    let (input_cells, output_cells) = util::load_self_cells_in_inputs_and_outputs()?;
    match action {
        b"create_reverse_record_root" => {}
        b"update_reverse_record_root" => {}
        _ => return Err(code_to_error!(ErrorCode::ActionNotSupported)),
    }

    Ok(())
}
