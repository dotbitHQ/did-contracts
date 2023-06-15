use alloc::boxed::Box;

use ckb_std::ckb_types::packed::Script;
use das_core::constants::das_lock;
use das_core::error::ScriptError;
use das_core::{assert, code_to_error};
use das_types::packed::{DeviceKeyList, DeviceKeyListCellData};
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::helpers::ToNum;
use crate::traits::{Action, FSMContract, Rule};

pub fn action() -> Action {
    let mut create_action = Action::new("create_device_key_list");
    create_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.input_inner_cells.len() == 0
                && contract.output_inner_cells.len() == 1
                && contract.output_inner_cells[0].0 == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 0 cell in input and 1 cell in output[0]"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify key length", |contract| {
        let key_list = contract.get_cell_witness::<DeviceKeyListCellData>(&contract.output_inner_cells[0])?;
        assert!(
            key_list.keys().item_count() == 1,
            ErrorCode::KeyListNumberIncorrect,
            "Should have exactly 1 key in key list"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify lock arg", |contract| {
        let key_list = contract.get_cell_witness::<DeviceKeyListCellData>(&contract.output_inner_cells[0])?;
        let mut lock_iter = contract.output_inner_cells.iter().map(|cell| cell.lock());
        let first_lock = lock_iter
            .next()
            .ok_or(code_to_error!(ErrorCode::InvalidTransactionStructure))?;
        assert!(
            lock_iter.all(|lock| lock.as_slice() == first_lock.as_slice()),
            ErrorCode::InvalidTransactionStructure,
            "All lock of the cell should be the same"
        );
        verify_key_list_lock_arg(&first_lock, key_list.keys())?;
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify output lock", |contract| {
        let lock = contract.output_inner_cells[0].lock();
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
            contract.output_inner_cells[0].capacity().to_num() >= 161 * 10u64.pow(8),
            ErrorCode::CapacityNotEnough,
            "There should be at least 161 CKB for capacity"
        );
        Ok(())
    }));

    create_action.add_verification(Rule::new("Verify refund lock", |contract| {
        let key_list = contract.get_cell_witness::<DeviceKeyListCellData>(&contract.output_inner_cells[0])?;
        let refund_lock = key_list.refund_lock();
        assert!(
            contract
                .input_outer_cells
                .iter()
                .all(|c| c.lock().as_slice() == refund_lock.as_slice())
                && contract
                    .output_outer_cells
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
