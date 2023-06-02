use alloc::boxed::Box;

use ckb_std::ckb_constants::Source;
use das_core::constants::ScriptType;
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, debug, util};
use das_types::constants::DataType;
use das_types::packed::{DeviceKey, DeviceKeyList};
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
        alloc::string::String::from_utf8(action.to_vec())
            .map_err(|_| das_core::error::ErrorCode::ActionNotSupported)?
    );

    // ensure cell deps
    parser.parse_cell()?;
    let config_main = parser.configs.main()?;
    let type_id_table = config_main.type_id_table();
    let device_key_list_cell_type_id = type_id_table.key_list_config_cell();
    // Ensure device_key_list_cell_contract is in cell_deps
    // ensure_unique_cell_deps([device_key_list_cell_type_id].as_slice())?;

    match action {
        b"create_device_key_list" => {
            das_core::assert!(
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Input)?.len() == 0,
                ErrorCode::FoundKeyListInInput,
                "There should be 0 device_key_list_cell in input "
            );

            let output_cells =
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Output)?;

            das_core::assert!(
                output_cells.len() == 1,
                ErrorCode::NoKeyListInOutput,
                "There should be exactly 1 device_key_list_cell in output"
            );

            let (_, _, bytes) = parser.verify_and_get(DataType::DeviceKeyList, output_cells[0], Source::Output)?;
            let key_list = DeviceKeyList::from_slice(bytes.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;

            das_core::assert!(
                key_list.len() == 1,
                ErrorCode::WitnessArgsInvalid,
                "There should be excatly 1 device_key when create"
            );

            verify_key_list_lock_arg(output_cells[0], key_list, Source::Output)?;
        }
        b"update_device_key_list" => {
            let input_cells =
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Input)?;
            let output_cells =
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Output)?;
            das_core::assert!(
                input_cells.len() == 1 && input_cells.len() == output_cells.len(),
                ErrorCode::InvalidTransactionStructure,
                "There should be exactly 1 device_key_list_cell in input and output"
            );

            das_core::assert!(
                ckb_std::high_level::load_cell_lock(input_cells[0], Source::Input)?
                    .args()
                    .as_slice()
                    == ckb_std::high_level::load_cell_lock(output_cells[0], Source::Output)?
                        .args()
                        .as_slice(),
                ErrorCode::InvalidLockArg,
                "Output lock arg should be the same as the one of the input"
            );

            let (_, _, key_list_in_input) =
                parser.verify_and_get(DataType::DeviceKeyList, input_cells[0], Source::Input)?;
            let (_, _, key_list_in_output) =
                parser.verify_and_get(DataType::DeviceKeyList, output_cells[0], Source::Output)?;
            let key_list_in_input = DeviceKeyList::from_slice(key_list_in_input.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;
            let key_list_in_output = DeviceKeyList::from_slice(key_list_in_output.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;

            das_core::assert!(
                key_list_in_output.item_count() > 0 && key_list_in_output.item_count() < 11,
                ErrorCode::UpdateParamsInvalid,
                "The key list length should be from 1 to 10"
            );

            let len_diff: i32 = key_list_in_input.len() as i32 - key_list_in_output.len() as i32;
            das_core::assert!(
                len_diff == 1 || len_diff == -1,
                ErrorCode::KeyListNumberIncorrect,
                "There should be exactly 1 device key difference when update"
            );

            let keys_in_input: alloc::collections::BTreeSet<DeviceKeyWrapped> =
                key_list_in_input.into_iter().map(|key| DeviceKeyWrapped(key)).collect();
            let keys_in_output: alloc::collections::BTreeSet<DeviceKeyWrapped> = key_list_in_output
                .into_iter()
                .map(|key| DeviceKeyWrapped(key))
                .collect();

            match len_diff {
                1 => {
                    // add device key
                    das_core::assert!(
                        keys_in_output.is_superset(&keys_in_input),
                        ErrorCode::UpdateParamsInvalid,
                        "Output keys should be superset of input"
                    );
                    let mut added_device_key = keys_in_output.difference(&keys_in_input);
                    let res = added_device_key.next().ok_or(ErrorCode::UpdateParamsInvalid)?;
                    das_core::assert!(
                        added_device_key.next().is_none(),
                        ErrorCode::UpdateParamsInvalid,
                        "Output key should be exactly 1 more than input"
                    );

                    res
                }
                -1 => {
                    // Remove device key
                    das_core::assert!(
                        keys_in_input.is_superset(&keys_in_output),
                        ErrorCode::UpdateParamsInvalid,
                        "Output keys should be superset of input"
                    );
                    let mut removed_device_key = keys_in_input.difference(&keys_in_output);
                    let res = removed_device_key.next().ok_or(ErrorCode::UpdateParamsInvalid)?;
                    das_core::assert!(
                        removed_device_key.next().is_none(),
                        ErrorCode::UpdateParamsInvalid,
                        "Output key should be exactly 1 less than input"
                    );
                    res
                }
                _ => unreachable!(),
            };
        },
        b"destroy_device_key_list" => {
            let input_cells =
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Input)?;
            let output_cells =
                util::find_cells_by_type_id(ScriptType::Type, device_key_list_cell_type_id, Source::Output)?;
            das_core::assert!(
                input_cells.len() == 1 && output_cells.len() == 0,
                ErrorCode::DestroyParamsInvalid,
                "Should have 1 key list in input and 0 in output"
            )
        },
        _ => unimplemented!(),
    }

    Ok(())
}

#[derive(Clone)]
struct DeviceKeyWrapped(DeviceKey);
impl Eq for DeviceKeyWrapped {}
impl PartialEq for DeviceKeyWrapped {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice() == other.0.as_slice()
    }
}

impl PartialOrd for DeviceKeyWrapped {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.as_slice().partial_cmp(&other.0.as_slice())
    }
}

impl Ord for DeviceKeyWrapped {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.as_slice().cmp(&other.0.as_slice())
    }
}

// fn ensure_unique_cell_dep(type_id: HashReader) -> Result<(), Box<dyn ScriptError>> {
//     let res = util::find_cells_by_type_id(ScriptType::Type, type_id, Source::CellDep)?;
//     if res.len() != 1 {
//         return Err(code_to_error!(ErrorCode::IndexOutOfBound));
//     }
//     Ok(())
// }

// fn ensure_unique_cell_deps(type_ids: &[HashReader]) -> Result<(), Box<dyn ScriptError>> {
//     for type_id in type_ids.into_iter() {
//         ensure_unique_cell_dep(*type_id)?;
//     }

//     Ok(())
// }

fn verify_key_list_lock_arg(index: usize, key_list: DeviceKeyList, source: Source) -> Result<(), Box<dyn ScriptError>> {
    let device_key = key_list.get(0).unwrap();
    let lock = ckb_std::high_level::load_cell_lock(index, source)?;
    let lock_arg = lock.args().raw_data();

    if lock_arg.len() != 44 {
        return Err(code_to_error!(ErrorCode::LockArgLengthIncorrect));
    }

    // First byte is main_alg_id
    das_core::assert!(
        lock_arg.slice(0..1) == device_key.main_alg_id().nth0().as_bytes(),
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
