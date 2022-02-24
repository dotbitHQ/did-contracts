use ckb_std::ckb_constants::Source;
use core::result::Result;
use das_core::{constants::*, debug, error::Error, util, warn, witness_parser::WitnessesParser};

pub fn main() -> Result<(), Error> {
    debug!("====== Running sub-account-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(Error::ActionNotSupported),
    };
    let action = action_cp.as_slice();

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| Error::ActionNotSupported)?
    );

    match action {
        b"enable_sub_account" => {
            util::require_type_script(
                &mut parser,
                TypeScript::AccountCellType,
                Source::Input,
                Error::InvalidTransactionStructure,
            )?;
        }
        b"create_sub_account" => {}
        b"edit_sub_account" => {}
        b"renew_sub_account" => {}
        b"recycle_sub_account" => {}
        _ => return Err(Error::ActionNotSupported),
    }

    Ok(())
}
