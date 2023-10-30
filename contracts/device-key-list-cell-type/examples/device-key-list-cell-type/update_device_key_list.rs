use alloc::vec::Vec;

use das_core::helpers::Comparable;
use das_core::traits::TryFromBytes;
use das_core::witness_parser::general_witness_parser::{get_witness_parser, EntityWrapper, ForNew, ForOld};
use das_core::{assert, code_to_error, debug};
use das_types::constants::DataType;
use das_types::packed::{DeviceKey, DeviceKeyListCellData};
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::helpers::ToNum;
use das_core::contract::defult_structs::{Action, Rule};

pub fn action() -> Action {
    let mut update_action = Action::new("update_device_key_list");
    update_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.get_input_outer_cells().len() == 0 && contract.get_output_outer_cells().len() == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should not have any balance cells in input or output"
        );
        assert!(
            contract.get_input_inner_cells().len() == 1
                && contract.get_output_inner_cells().len() == 1
                && contract.get_input_inner_cells()[0].meta.index == 0
                && contract.get_output_inner_cells()[0].meta.index == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 1 device_key_list_cell in input[0] and 1 cell in output[0]"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify capacity change", |contract| {
        assert!(
            i64::try_from(contract.get_input_inner_cells()[0].capacity().to_num()).unwrap()
                - i64::try_from(contract.get_output_inner_cells()[0].capacity().to_num()).unwrap()
                <= 10000,
            ErrorCode::CapacityReduceTooMuch,
            "Capacity change is too much"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify lock consistent", |contract| {
        assert!(
            contract.get_input_inner_cells()[0].lock().as_slice()
                == contract.get_output_inner_cells()[0].lock().as_slice(),
            ErrorCode::InvalidLock,
            "Lock should not change"
        );
        Ok(())
    }));

    update_action.add_verification(Rule::new("Verify key list structure", |contract| {
        let input_cell_meta = contract.get_input_inner_cells()[0].get_meta();
        let output_cell_meta = contract.get_output_inner_cells()[0].get_meta();
        let key_list_in_input = get_witness_parser()
            .parse_for_cell::<EntityWrapper<{ DataType::DeviceKeyListEntityData as u32 }, ForOld>>(input_cell_meta)?
            .result
            .into_target()
            .map(|e| DeviceKeyListCellData::try_from_bytes(e.entity().raw_data()))
            .unwrap()?;
        let key_list_in_output = get_witness_parser()
            .parse_for_cell::<EntityWrapper<{ DataType::DeviceKeyListEntityData as u32 }, ForNew>>(output_cell_meta)?
            .result
            .into_target()
            .map(|e| DeviceKeyListCellData::try_from_bytes(e.entity().raw_data()))
            .unwrap()?;

        assert!(
            key_list_in_input.refund_lock().as_slice() == key_list_in_output.refund_lock().as_slice(),
            ErrorCode::UpdateParamsInvalid,
            "Changes to refund_lock are not allowed"
        );

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
