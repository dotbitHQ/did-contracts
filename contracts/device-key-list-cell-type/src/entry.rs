use alloc::boxed::Box;
use alloc::vec::Vec;

use ckb_std::ckb_constants::{Source};
use ckb_std::ckb_types::packed::Script;
use ckb_std::high_level::{load_cell, load_cell_lock, QueryIter};
use das_core::constants::{das_lock, ScriptHashType, ScriptType};
use das_core::error::ScriptError;
use das_core::witness_parser::WitnessesParser;
use das_core::{code_to_error, debug, util, verifiers};
use das_types::constants::DataType;
use das_types::packed::{DeviceKey, DeviceKeyList, DeviceKeyListCellData};
use das_types::prelude::Entity;

use crate::error::ErrorCode;
use crate::helpers::Comparable;

pub fn main() -> Result<(), Box<dyn ScriptError>> {
    debug!("====== Running sub-account-cell-type ======");
    let mut parser = WitnessesParser::new()?;
    let action = match parser.parse_action_with_params()? {
        Some((action, _)) => action.to_vec(),
        None => return Err(code_to_error!(das_core::error::ErrorCode::ActionNotSupported)),
    };

    debug!(
        "Route to {:?} action ...",
        alloc::string::String::from_utf8(action.clone()).map_err(|_| das_core::error::ErrorCode::ActionNotSupported)?
    );

    parser.parse_cell()?;
    let this_script = ckb_std::high_level::load_script()?;

    match action.as_slice() {
        b"create_device_key_list" => {
            let (input_cells, output_cells) =
                util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_script.as_reader())?;

            verifiers::common::verify_cell_number_and_position(
                "device-key-list",
                &input_cells,
                &[],
                &output_cells,
                &[0],
            )?;

            let (_, _, bytes) = parser.verify_and_get(DataType::DeviceKeyList, output_cells[0], Source::Output)?;
            let key_list_cell_data = DeviceKeyListCellData::from_compatible_slice(bytes.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;

            das_core::assert!(
                key_list_cell_data.keys().item_count() == 1,
                ErrorCode::WitnessArgsInvalid,
                "There should be excatly 1 device_key when create"
            );

            let cell_lock = ckb_std::high_level::load_cell_lock(output_cells[0], Source::Output)?;
            verify_key_list_lock_arg(&cell_lock, key_list_cell_data.keys())?;
            das_core::assert!(
                cell_lock.hash_type() == ScriptHashType::Type.into()
                    && cell_lock.code_hash().as_slice() == das_lock().code_hash().as_slice(),
                ErrorCode::MustUseDasLock,
                "Output device-key-list-cell must use das-lock"
            );

            let capacity = ckb_std::high_level::load_cell_capacity(output_cells[0], Source::Output)?;
            das_core::assert!(
                capacity > 161 * 10u64.pow(8),
                ErrorCode::CapacityNotEnough,
                "There should be at least 160 CKB for capacity"
            );

            let mut input_cell_locks = QueryIter::new(load_cell, Source::Input).map(|c| c.lock());
            let mut output_cell_locks =
                QueryIter::new(load_cell, Source::Output).filter_map(|c| match c.type_().to_opt() {
                    Some(t) if t.code_hash().as_slice() == this_script.code_hash().as_slice() => None,
                    _ => Some(c.lock()),
                });

            let refund_lock = key_list_cell_data.refund_lock();

            das_core::assert!(
                input_cell_locks.all(|l| l.as_slice() == refund_lock.as_slice())
                    && output_cell_locks.all(|l| l.as_slice() == refund_lock.as_slice()),
                ErrorCode::InconsistentBalanceCellLocks,
                "All locks for balance-cell should be the same"
            );
        }
        b"update_device_key_list" => {
            let (input_cells, output_cells) =
                util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_script.as_reader())?;
            verifiers::common::verify_cell_number_and_position(
                "device-key-list",
                &input_cells,
                &[0],
                &output_cells,
                &[0],
            )?;

            das_core::assert!(
                ckb_std::high_level::load_cell_capacity(input_cells[0], Source::Input)?
                    - ckb_std::high_level::load_cell_capacity(output_cells[0], Source::Output)?
                    < 10000,
                ErrorCode::CapacityReduceTooMuch,
                "Capacity change is too much"
            );

            das_core::assert!(
                ckb_std::high_level::load_cell_lock(input_cells[0], Source::Input)?.as_slice()
                    == ckb_std::high_level::load_cell_lock(output_cells[0], Source::Output)?.as_slice(),
                ErrorCode::InvalidLock,
                "Lock should not change"
            );

            let (_, _, key_list_in_input) =
                parser.verify_and_get(DataType::DeviceKeyList, input_cells[0], Source::Input)?;
            let (_, _, key_list_in_output) =
                parser.verify_and_get(DataType::DeviceKeyList, output_cells[0], Source::Output)?;
            let key_list_in_input = DeviceKeyListCellData::from_compatible_slice(key_list_in_input.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?
                .keys();
            let key_list_in_output = DeviceKeyListCellData::from_compatible_slice(key_list_in_output.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?
                .keys();

            das_core::assert!(
                key_list_in_output.item_count() > 0 && key_list_in_output.item_count() < 11,
                ErrorCode::UpdateParamsInvalid,
                "The key list length should be from 1 to 10"
            );

            let len_diff: i32 = key_list_in_input.item_count() as i32 - key_list_in_output.item_count() as i32;
            das_core::assert!(
                len_diff == 1 || len_diff == -1,
                ErrorCode::KeyListNumberIncorrect,
                "There should be exactly 1 device key difference when update"
            );

            match len_diff {
                1 => {
                    debug!("update_device_key_list: add key");
                    // Should only append to the tail
                    let mut input_iter = key_list_in_input.into_iter();
                    let mut output_iter = key_list_in_output.into_iter();
                    loop {
                        match (input_iter.next(), output_iter.next()) {
                            (Some(a), Some(b)) if a.as_slice() == b.as_slice() => continue,
                            (Some(_), Some(_)) => Err(code_to_error!(ErrorCode::UpdateParamsInvalid))?,
                            (None, Some(_)) => break,
                            _ => unreachable!(),
                        }
                    }
                }
                -1 => {
                    debug!("update_device_key_list: remove key");
                    let keys_in_input: alloc::collections::BTreeSet<Comparable<DeviceKey>> =
                        key_list_in_input.into_iter().map(|key| Comparable(key)).collect();
                    let keys_in_output: alloc::collections::BTreeSet<Comparable<DeviceKey>> = key_list_in_output
                        .into_iter()
                        .map(|key| Comparable(key))
                        .collect();
                    das_core::assert!(
                        keys_in_input.is_superset(&keys_in_output),
                        ErrorCode::UpdateParamsInvalid,
                        "Output keys should be superset of input"
                    );
                    let removed_device_key: Vec<Comparable<DeviceKey>> =
                        keys_in_input.difference(&keys_in_output).cloned().collect();
                    das_core::assert!(
                        removed_device_key.len() == 1,
                        ErrorCode::UpdateParamsInvalid,
                        "Output key should be exactly 1 less than input"
                    );
                }
                _ => unreachable!(),
            };
        }
        b"destroy_device_key_list" => {
            let (input_cells, output_cells) =
                util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, this_script.as_reader())?;
            verifiers::common::verify_cell_number_and_position(
                "device-key-list",
                &input_cells,
                &[0],
                &output_cells,
                &[],
            )?;
            let (_, _, key_list_in_input) =
                parser.verify_and_get(DataType::DeviceKeyList, input_cells[0], Source::Input)?;
            let key_list_in_input = DeviceKeyListCellData::from_compatible_slice(key_list_in_input.as_slice())
                .map_err(|_e| code_to_error!(ErrorCode::KeyListParseError))?;
            let mut output_cell_locks = QueryIter::new(load_cell_lock, Source::Output);
            das_core::assert!(
                output_cell_locks.all(|l| l.as_slice() == key_list_in_input.refund_lock().as_slice()),
                ErrorCode::InconsistentBalanceCellLocks,
                "Should return capacity to refund_lock"
            );
        }
        _ => unimplemented!(),
    }

    Ok(())
}


// TODO: refactor the logic into common verifiers.
fn verify_key_list_lock_arg(lock: &Script, key_list: DeviceKeyList) -> Result<(), Box<dyn ScriptError>> {
    let device_key = key_list.get(0).unwrap();
    let lock_arg = lock.args().raw_data();

    if lock_arg.len() != 44 {
        return Err(code_to_error!(ErrorCode::LockArgLengthIncorrect));
    }

    // First byte is main_alg_id
    das_core::assert!(
        lock_arg.slice(0..1) == device_key.main_alg_id().nth0().as_bytes(),
        ErrorCode::InvalidLock,
        "First byte of lock arg should be main_alg_id"
    );

    // Second byte is sub_alg_id
    das_core::assert!(
        lock_arg.slice(1..2) == device_key.sub_alg_id().nth0().as_bytes(),
        ErrorCode::InvalidLock,
        "Second byte of lock arg should be sub_alg_id"
    );

    // Next 10 bytes are pubkey hashed 5 times
    das_core::assert!(
        lock_arg.slice(2..12) == device_key.pubkey().raw_data(),
        ErrorCode::InvalidLock,
        "Byte 2..12 should be pubkey'"
    );

    // Next 10 bytes are cid hashed 5 times
    das_core::assert!(
        lock_arg.slice(12..22) == device_key.cid().raw_data(),
        ErrorCode::InvalidLock,
        "Byte 12..22 should be cid'"
    );

    // Owner and manager are the same
    das_core::assert!(
        lock_arg.slice(0..22) == lock_arg.slice(22..44),
        ErrorCode::InvalidLock,
        "Byte 0..22 should be the same with Byte 22..44"
    );

    Ok(())
}
