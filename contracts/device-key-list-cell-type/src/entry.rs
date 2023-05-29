use alloc::boxed::Box;
use ckb_std::ckb_constants::Source;
use das_core::constants::ScriptType;
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, debug, util};
use das_types::constants::DataType;
use das_types::packed::{DeviceKeyList, HashReader};
use das_types::prelude::Entity;

use crate::error::ErrorCode;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running sub-account-cell-type ======");
    let mut parser = WitnessesParser::new()?;
    let action_cp = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(das_core::error::ErrorCode::ActionNotSupported)),
    };
    let action = action_cp.as_slice();

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.to_vec()).map_err(|_| das_core::error::ErrorCode::ActionNotSupported)?
    );

    // ensure cell deps
    parser.parse_cell()?;
    let config_main = parser.configs.main()?;
    let type_id_table = config_main.type_id_table();
    let device_key_list_cell_type_id = type_id_table.key_list_config_cell();
    // Ensure device_key_list_cell_contract is in cell_deps
    ensure_unique_cell_deps([device_key_list_cell_type_id].as_slice())?;

    match action {
        b"create_key_list" => {
            das_core::assert!(
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::GroupInput)?.len()
                    == 0,
                ErrorCode::FoundKeyListInInput,
                "There should be 0 device_key_list_cell in input "
            );

            // Intentionally do not check the number of output device_key_list for future batch create
            let key_list_in_output =
                parser.verify_and_get_list_from_witness(DataType::DeviceKeyList, Source::GroupOutput)?;

            das_core::assert!(
                key_list_in_output.len() > 0,
                ErrorCode::NoKeyListInOutput,
                "There should be at least 1 device_key_list_cell in output"
            );

            fn validate_key_list(index: usize, key_list: DeviceKeyList) -> Result<(), Box<dyn ScriptError>> {
                if key_list.len() != 1 {
                    return Err(code_to_error!(ErrorCode::WitnessArgsInvalid));
                }

                let device_key = key_list.get(0).unwrap();
                let lock = ckb_std::high_level::load_cell_lock(index, Source::GroupOutput)?;
                let lock_arg = lock.args().raw_data();

                if lock_arg.len() != 44 {
                    return Err(code_to_error!(ErrorCode::LockArgLengthIncorrect));
                }

                // First byte is main_alg_id
                das_core::assert!(
                    lock_arg.slice(0..1)== device_key.main_alg_id().nth0().as_bytes(),
                    ErrorCode::InvalidLockArg,
                    "First byte of lock arg should be main_alg_id"
                );

                // Second byte is sub_alg_id
                das_core::assert!(
                    lock_arg.slice(1..2) == device_key.sub_alg_id().nth0().as_bytes(),
                    ErrorCode::InvalidLockArg,
                    "Second byte of lock arg should be sub_alg_id"
                );

                // Next 10 bytes are pubkey hashed 5 times
                das_core::assert!(
                    lock_arg.slice(2..12) == device_key.pubkey().raw_data(),
                    ErrorCode::InvalidLockArg,
                    "Byte 2..12 should be pubkey'"
                );

                // Next 10 bytes are cid hashed 5 times
                das_core::assert!(
                    lock_arg.slice(12..22) == device_key.cid().raw_data(),
                    ErrorCode::InvalidLockArg,
                    "Byte 12..22 should be cid'"
                );

                // Owner and manager are the same
                das_core::assert!(
                    lock_arg.slice(0..22) == lock_arg.slice(22..44),
                    ErrorCode::InvalidLockArg,
                    "Byte 0..22 should be the same with Byte 22..44"
                );

                Ok(())
            }

            for (index, _, bytes) in key_list_in_output {
                let key_list = DeviceKeyList::from_slice(bytes.as_slice()).map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;
                validate_key_list(index as usize, key_list)?;
            }
        },
        _ => unimplemented!()
    }

    Ok(())
}

fn ensure_unique_cell_dep(type_id: HashReader) -> Result<(), Box<dyn ScriptError>> {
    let res = util::find_cells_by_type_id(ScriptType::Type, type_id, Source::CellDep)?;
    if res.len() != 1 {
        return Err(code_to_error!(ErrorCode::IndexOutOfBound));
    }
    Ok(())
}

fn ensure_unique_cell_deps(type_ids: &[HashReader]) -> Result<(), Box<dyn ScriptError>> {
    for type_id in type_ids.into_iter() {
        ensure_unique_cell_dep(*type_id)?;
    }

    Ok(())
}
