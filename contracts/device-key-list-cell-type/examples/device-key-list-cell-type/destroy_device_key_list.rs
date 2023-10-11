use das_core::general_witness_parser::{get_witness_parser, EntityWrapper, ForOld};
use das_core::{assert, code_to_error};
use das_types::packed::DeviceKeyListCellData;
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::helpers::ToNum;
use crate::traits::{Action, Rule};

pub fn action() -> Action {
    let mut destroy_action = Action::new("destroy_device_key_list");
    destroy_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.get_input_inner_cells().len() == 1
                && contract.get_output_inner_cells().len() == 0
                && contract.get_input_inner_cells()[0].meta.index == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 1 cell in input[0] and 0 cell in output"
        );
        Ok(())
    }));

    destroy_action.add_verification(Rule::new("Verify refund lock", |contract| {
        let input_cell_meta = contract.get_input_inner_cells()[0].get_meta();
        let key_list_in_input: DeviceKeyListCellData = get_witness_parser()
            .parse_for_cell::<EntityWrapper<DeviceKeyListCellData, ForOld>>(input_cell_meta)?
            .result
            .into_inner()
            .unwrap();
        let refund_lock = key_list_in_input.refund_lock();
        assert!(
            contract
                .get_output_outer_cells()
                .iter()
                .all(|c| c.lock().as_slice() == refund_lock.as_slice()),
            ErrorCode::InconsistentBalanceCellLocks,
            "Should return capacity to refund_lock"
        );
        Ok(())
    }));

    destroy_action.add_verification(Rule::new("Check total capacity change", |contract| {
        let input_capacity: u64 = contract
            .get_input_inner_cells()
            .iter()
            .map(|cell| cell.capacity().to_num())
            .sum::<u64>()
            + contract
                .get_input_outer_cells()
                .iter()
                .map(|cell| cell.capacity().to_num())
                .sum::<u64>();

        let output_capacity: u64 = contract
            .get_output_inner_cells()
            .iter()
            .map(|cell| cell.capacity().to_num())
            .sum::<u64>()
            + contract
                .get_output_outer_cells()
                .iter()
                .map(|cell| cell.capacity().to_num())
                .sum::<u64>();
        assert!(
            input_capacity - output_capacity <= 10000,
            ErrorCode::CapacityReduceTooMuch,
            "Should not pay too much to miner"
        );

        Ok(())
    }));

    destroy_action
}
