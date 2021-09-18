use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, error::SysError, high_level};
use das_core::{
    constants::DasLockType,
    constants::{das_lock, ScriptType},
    debug,
    eip712::verify_eip712_hashes,
    error::Error,
    util, warn,
    witness_parser::WitnessesParser,
};
use das_types::{constants::DataType, packed as das_packed, prelude::*};

pub fn main() -> Result<(), Error> {
    debug!("====== Running balance-cell-type ======");

    let mut parser = WitnessesParser::new()?;
    let action_opt = parser.parse_action_with_params()?;

    if action_opt.is_none() {
        return Err(Error::ActionNotSupported);
    }

    let (action_raw, params_raw) = action_opt.unwrap();
    let _action = action_raw.as_reader().raw_data();
    let params = params_raw.iter().map(|param| param.as_reader()).collect::<Vec<_>>();

    parser.parse_config(&[DataType::ConfigCellMain])?;
    parser.parse_cell()?;

    verify_eip712_hashes(&parser, action_raw.as_reader(), &params)?;

    debug!("Check if any cell in outputs with das-lock lack of the type script named balance-cell-type or account-cell-type.");

    let account_cell_type_id = parser.configs.main()?.type_id_table().account_cell();
    let account_cell_type = das_packed::Script::new_builder()
        .code_hash(account_cell_type_id.to_entity())
        .hash_type(das_packed::Byte::new(ScriptType::Type as u8))
        .build();
    let this_script = high_level::load_script().map_err(|e| Error::from(e))?;
    let this_script_reader = this_script.as_reader();

    let das_lock = das_lock();
    let das_lock_reader = das_lock.as_reader();
    let mut i = 0;
    loop {
        let ret = high_level::load_cell_lock(i, Source::Output);
        match ret {
            Ok(lock) => {
                let lock_reader = lock.as_reader();
                // Check if cells with das-lock in outputs also has the type script named balance-cell-type or account-cell-type.
                if util::is_script_equal(das_lock_reader, lock_reader) {
                    let type_of_lock = lock_reader.args().raw_data()[0];
                    if type_of_lock == DasLockType::ETHTypedData as u8 {
                        let type_opt = high_level::load_cell_type(i, Source::Output).map_err(|e| Error::from(e))?;
                        match type_opt {
                            Some(type_) if util::is_reader_eq(this_script_reader, type_.as_reader()) => {}
                            Some(type_)
                                if util::is_reader_eq(account_cell_type.as_reader(), type_.as_reader().into()) => {}
                            _ => {
                                warn!("Outputs[{}] This cell has das-lock, so it should also has the type script named balance-cell-type or account-cell-type.", i);
                                return Err(Error::BalanceCellFoundSomeOutputsLackOfType);
                            }
                        }
                    }
                }
            }
            Err(SysError::IndexOutOfBound) => {
                break;
            }
            Err(err) => {
                return Err(Error::from(err));
            }
        }

        i += 1;
    }

    Ok(())
}
