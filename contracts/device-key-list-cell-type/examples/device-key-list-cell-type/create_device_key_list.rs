use alloc::boxed::Box;

use ckb_std::ckb_types::packed::Script;
use das_core::constants::das_lock;
use das_core::contract::defult_structs::{Action, Rule};
use das_core::error::ScriptError;
use das_core::traits::TryFromBytes;
use das_core::witness_parser::general_witness_parser::{get_witness_parser, EntityWrapper, ForNew};
use das_core::{assert, code_to_error};
use das_types::constants::DataType;
use das_types::packed::{DeviceKeyList, DeviceKeyListCellData};
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::helpers::ToNum;

pub fn action() -> Action {
    let mut create_action = Action::new("create_device_key_list");
    create_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.get_input_inner_cells().len() == 0
                && contract.get_output_inner_cells().len() == 1
                && contract.get_output_inner_cells()[0].meta.index == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 0 cell in input and 1 cell in output[0]"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify key length", |contract| {
        let output_cell_meta = contract.get_output_inner_cells()[0].get_meta();
        let key_list = get_witness_parser()
            .parse_for_cell::<EntityWrapper<{ DataType::DeviceKeyListEntityData as u32 }, ForNew>>(output_cell_meta)?
            .result
            .into_target()
            .map(|e| DeviceKeyListCellData::try_from_bytes(e.entity().raw_data()))
            .unwrap()?;
        assert!(
            key_list.keys().item_count() == 1,
            ErrorCode::KeyListNumberIncorrect,
            "Should have exactly 1 key in key list"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("The lock arg of key list should be ", |contract| {
        let output_cell_meta = contract.get_output_inner_cells()[0].get_meta();
        let key_list = get_witness_parser()
            .parse_for_cell::<EntityWrapper<{ DataType::DeviceKeyListEntityData as u32 }, ForNew>>(output_cell_meta)?
            .result
            .into_target()
            .map(|e| DeviceKeyListCellData::try_from_bytes(e.entity().raw_data()))
            .unwrap()?;
        verify_key_list_lock_arg(&contract.get_output_inner_cells()[0].lock(), key_list.keys())?;
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify output lock", |contract| {
        let lock = contract.get_output_inner_cells()[0].lock();
        let das_lock = das_lock();
        assert!(
            lock.hash_type().as_slice() == das_lock.hash_type().as_slice()
                && lock.code_hash().as_slice() == das_lock.code_hash().as_slice(),
            ErrorCode::InvalidLock,
            "Output device-key-list-cell must use das-lock"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify capacity", |contract| {
        assert!(
            contract.get_output_inner_cells()[0].capacity().to_num() >= 161 * 10u64.pow(8),
            ErrorCode::CapacityNotEnough,
            "There should be at 161 CKB base capacity for key-list-cell (output[0])"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify refund lock", |contract| {
        let output_cell_meta = contract.get_output_inner_cells()[0].get_meta();
        let key_list = get_witness_parser()
            .parse_for_cell::<EntityWrapper<{ DataType::DeviceKeyListEntityData as u32 }, ForNew>>(output_cell_meta)?
            .result
            .into_target()
            .map(|e| DeviceKeyListCellData::try_from_bytes(e.entity().raw_data()))
            .unwrap()?;
        let refund_lock = key_list.refund_lock();
        assert!(
            contract
                .get_input_outer_cells()
                .iter()
                .all(|c| c.lock().as_slice() == refund_lock.as_slice())
                && contract
                    .get_output_outer_cells()
                    .iter()
                    .all(|c| c.lock().as_slice() == refund_lock.as_slice()),
            ErrorCode::InconsistentBalanceCellLocks,
            "All locks for balance-cell should be the same"
        );
        Ok(())
    }));

    create_action
}

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

    // Next 10 bytes are cid hashed 5 times
    das_core::assert!(
        lock_arg.slice(2..12) == device_key.cid().raw_data(),
        ErrorCode::InvalidLock,
        "Byte 2..12 should be cid'"
    );

    // Next 10 bytes are pubkey hashed 5 times
    das_core::assert!(
        lock_arg.slice(12..22) == device_key.pubkey().raw_data(),
        ErrorCode::InvalidLock,
        "Byte 12..22 should be pubkey'"
    );

    // Owner and manager are the same
    das_core::assert!(
        lock_arg.slice(0..22) == lock_arg.slice(22..44),
        ErrorCode::InvalidLock,
        "Byte 0..22 should be the same with Byte 22..44"
    );

    Ok(())
}
