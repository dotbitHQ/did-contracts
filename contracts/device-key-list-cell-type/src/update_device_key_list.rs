use alloc::boxed::Box;
use alloc::vec::Vec;

use ckb_std::ckb_types::packed::Script;
use das_core::constants::das_lock;
use das_core::error::ScriptError;
use das_core::{assert, code_to_error, debug};
use das_types::convert::*;
use das_types::packed::{DeviceKey, DeviceKeyList, DeviceKeyListCellData};
use molecule::prelude::Entity;

use crate::error::ErrorCode;
use crate::helpers::{Comparable, ToNum};
use crate::traits::{Action, FSMContract, Rule};

pub fn action() -> Action {
    let mut update_action = Action::new("update_device_key_list");
    update_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.input_inner_cells.len() == 1
                && contract.output_inner_cells.len() == 1
                && contract.input_inner_cells[0].0 == 0
                && contract.output_inner_cells[0].0 == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 1 cell in input[0] and 1 cell in output[0]"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify capacity change", |contract| {
        assert!(
            i64::try_from(contract.input_inner_cells[0].capacity().to_num()).unwrap()
                - i64::try_from(contract.output_inner_cells[0].capacity().to_num()).unwrap()
                < 10000,
            ErrorCode::CapacityReduceTooMuch,
            "Capacity change is too much"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify lock consistent", |contract| {
        assert!(
            contract.input_inner_cells[0].lock().as_slice() == contract.output_inner_cells[0].lock().as_slice(),
            ErrorCode::InvalidLock,
            "Lock should not change"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify key list structure", |contract| {
        let key_list_in_input = contract.get_cell_witness::<DeviceKeyListCellData>(&contract.input_inner_cells[0])?;
        let key_list_in_output = contract.get_cell_witness::<DeviceKeyListCellData>(&contract.output_inner_cells[0])?;
        das_core::assert!(
            key_list_in_output.keys().item_count() > 0 && key_list_in_output.keys().item_count() < 11,
            ErrorCode::UpdateParamsInvalid,
            "The key list length should be from 1 to 10"
        );

        let len_diff: i64 = i64::try_from(key_list_in_output.keys().item_count()).unwrap()
            - i64::try_from(key_list_in_input.keys().item_count()).unwrap();
        das_core::assert!(
            len_diff == 1 || len_diff == -1,
            ErrorCode::KeyListNumberIncorrect,
            "There should be exactly 1 device key difference when update"
        );
        let keys_in_input: alloc::collections::BTreeSet<Comparable<DeviceKey>> = key_list_in_input
            .keys()
            .clone()
            .into_iter()
            .map(|key| Comparable(key))
            .collect();
        let keys_in_output: alloc::collections::BTreeSet<Comparable<DeviceKey>> = key_list_in_output
            .keys()
            .clone()
            .into_iter()
            .map(|key| Comparable(key))
            .collect();

        match len_diff {
            1 => {
                debug!("update_device_key_list: add key");
                // Should only append to the tail
                let mut input_iter = key_list_in_input.keys().into_iter();
                let mut output_iter = key_list_in_output.keys().clone().into_iter();
                loop {
                    match (input_iter.next(), output_iter.next()) {
                        (Some(a), Some(b)) if a.as_slice() == b.as_slice() => continue,
                        (Some(_), Some(_)) => Err(code_to_error!(ErrorCode::UpdateParamsInvalid))?,
                        (None, Some(_)) => break,
                        _ => unreachable!(),
                    }
                }
                das_core::assert!(
                    keys_in_output.len() == key_list_in_output.keys().item_count(),
                    ErrorCode::DuplicatedKeys,
                    "Should not add duplicated keys"
                );
            }
            -1 => {
                debug!("update_device_key_list: remove key");
                das_core::assert!(
                    keys_in_input.is_superset(&keys_in_output),
                    ErrorCode::UpdateParamsInvalid,
                    "Output keys should be subset of input"
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
        Ok(())
    }));

    update_action
}
