use crate::{assert, debug, error::Error};
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, high_level};

pub fn verify_created_cell_in_correct_position(
    cell_name: &str,
    input_cell_indexes: &[usize],
    output_cell_indexes: &[usize],
    created_cell_index: Option<usize>, // None: No need to verify the index of the created cell; Some(index): the created cell should be at index, usually 1
) -> Result<(), Error> {
    assert!(
        input_cell_indexes.len() == 0 && output_cell_indexes.len() == 1,
        Error::InvalidTransactionStructure,
        "To create cell, there should be 0 {} in inputs and 1 {} in outputs, while there is actually {} in inputs and {} in outputs.",
        cell_name,
        cell_name,
        input_cell_indexes.len(),
        output_cell_indexes.len()
    );
    if let Some(index) = created_cell_index {
        assert!(
            output_cell_indexes[0] == index,
            Error::InvalidTransactionStructure,
            "To create cell, the {} should only appear at outputs[1], it actually appear at {}",
            cell_name,
            output_cell_indexes[0]
        );
    }

    Ok(())
}

pub fn verify_removed_cell_in_correct_position(
    cell_name: &str,
    input_cell_indexes: &[usize],
    output_cell_indexes: &[usize],
    removed_cell_index: Option<usize>, // None: No need to verify the index of the removed cell; Some(index): the removed cell should be at index, usually 1
) -> Result<(), Error> {
    assert!(
        input_cell_indexes.len() == 1 && output_cell_indexes.len() == 0,
        Error::InvalidTransactionStructure,
        "To remove cell, there should be 1 {} in inputs and 0 {} in outputs. (received inputs: {}, outputs: {})",
        cell_name,
        cell_name,
        input_cell_indexes.len(),
        output_cell_indexes.len()
    );

    if let Some(index) = removed_cell_index {
        assert!(
            input_cell_indexes[0] == index,
            Error::InvalidTransactionStructure,
            "To remove cell, the {} should only appear at inputs[1], while received {}",
            cell_name,
            input_cell_indexes[0]
        );
    }

    Ok(())
}

pub fn verify_modified_cell_in_correct_position(
    cell_name: &str,
    input_cells: &[usize],
    output_cells: &[usize],
) -> Result<(), Error> {
    assert!(
        input_cells.len() == 1 && output_cells.len() == 1,
        Error::InvalidTransactionStructure,
        "To modify {}, there should be 1 {} in inputs and 1 {} in outputs, while received {} in inputs and {} in outputs",
        cell_name,
        cell_name,
        cell_name,
        input_cells.len(),
        output_cells.len()
    );
    assert!(
        input_cells[0] == 0 && output_cells[0] == 0,
        Error::InvalidTransactionStructure,
        "To modify {}, the {} should only appear at inputs[0] and outputs[0], while received {} and {}",
        cell_name,
        cell_name,
        input_cells[0],
        output_cells[0]
    );

    Ok(())
}

// The tx fee is from a specific cell (like AccountCell/AccountSaleCell), here to verify the validity of the fee spent
pub fn verify_tx_fee_spent_correctly(
    cell_name: &str,
    input_cell: usize,
    output_cell: usize,
    expected_fee: u64,
    basic_capacity: u64,
) -> Result<(), Error> {
    debug!("Verify if {} paid fee correctly.", cell_name);

    let input_capacity = high_level::load_cell_capacity(input_cell, Source::Input)?;
    let output_capacity = high_level::load_cell_capacity(output_cell, Source::Output)?;

    if input_capacity > output_capacity {
        assert!(
            output_capacity >= basic_capacity,
            Error::TxFeeSpentError, // changed from Error::AccountSaleCellFeeError
            "The {} has no more capacity as fee for this transaction.(input_capacity: {}, output_capacity: {}, basic_capacity: {})",
            cell_name,
            input_capacity,
            output_capacity,
            basic_capacity
        );

        assert!(
            input_capacity - output_capacity <= expected_fee, //  output_capacity >= input_capacity - expected_fee,
            Error::TxFeeSpentError,
            "The {} fee should be equal to or less than {}, result: {})",
            cell_name,
            expected_fee,
            input_capacity - output_capacity
        );
    } else if input_capacity == output_capacity {
        debug!(
            "The capacity of {} didn't change, which user pay the fee himself. That's ok.",
            cell_name
        );
    } else {
        debug!(
            "User put more capacity into {}, input: {} < output: {}. That's ok.",
            cell_name, input_capacity, output_capacity
        );
    }

    Ok(())
}
