use crate::{
    assert as das_assert,
    constants::{das_wallet_lock, ScriptType},
    debug,
    error::Error,
    util,
};
use alloc::format;
use ckb_std::{ckb_constants::Source, high_level};
use core::cmp::Ordering;

pub fn verify_cell_dep_number(cell_name: &str, current_deps: &[usize], expected_deps_len: usize) -> Result<(), Error> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_deps.len() == expected_deps_len,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_deps_len {
            0 => format!("There should be none {} in inputs.", cell_name),
            1 => format!("There should be only one {} in inputs.", cell_name),
            _ => format!("There should be {} {}s in inputs.", expected_deps_len, cell_name),
        }
    );

    Ok(())
}

pub fn verify_cell_number(
    cell_name: &str,
    current_inputs: &[usize],
    expected_inputs_len: usize,
    current_outputs: &[usize],
    expected_outputs_len: usize,
) -> Result<(), Error> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_inputs.len() == expected_inputs_len,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_inputs_len {
            0 => format!("There should be none {} in inputs.", cell_name),
            1 => format!("There should be only one {} in inputs.", cell_name),
            _ => format!("There should be {} {}s in inputs.", expected_inputs_len, cell_name),
        }
    );

    das_assert!(
        current_outputs.len() == expected_outputs_len,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_outputs_len {
            0 => format!("There should be none {} in outputs.", cell_name),
            1 => format!("There should be only one {} in outputs.", cell_name),
            _ => format!("There should be {} {}s in outputs.", expected_outputs_len, cell_name),
        }
    );

    Ok(())
}

pub fn verify_cell_number_range(
    cell_name: &str,
    current_inputs: &[usize],
    expected_inputs_range: (Ordering, usize),
    current_outputs: &[usize],
    expected_outputs_range: (Ordering, usize),
) -> Result<(), Error> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_inputs.len().cmp(&expected_inputs_range.1) == expected_inputs_range.0,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_inputs_range.0 {
            Ordering::Less => format!(
                "There should be less than {} {}s in inputs.",
                expected_inputs_range.1, cell_name
            ),
            Ordering::Greater => format!(
                "There should be more than {} {}s in inputs.",
                expected_inputs_range.1, cell_name
            ),
            Ordering::Equal => format!(
                "There should be exactly {} {}s in inputs.",
                expected_inputs_range.1, cell_name
            ),
        }
    );

    das_assert!(
        current_outputs.len().cmp(&expected_outputs_range.1) == expected_outputs_range.0,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_outputs_range.0 {
            Ordering::Less => format!(
                "There should be less than {} {}s in inputs.",
                expected_outputs_range.1, cell_name
            ),
            Ordering::Greater => format!(
                "There should be more than {} {}s in inputs.",
                expected_outputs_range.1, cell_name
            ),
            Ordering::Equal => format!(
                "There should be exactly {} {}s in inputs.",
                expected_outputs_range.1, cell_name
            ),
        }
    );

    Ok(())
}

pub fn verify_cell_number_and_position(
    cell_name: &str,
    current_inputs: &[usize],
    expected_inputs: &[usize],
    current_outputs: &[usize],
    expected_outputs: &[usize],
) -> Result<(), Error> {
    debug!("Verify if the number and position of {}s is correct.", cell_name);

    das_assert!(
        current_inputs == expected_inputs,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_inputs.len() {
            0 => format!("There should be none {} in inputs.", cell_name),
            1 => format!(
                "There should be only one {} in inputs[{}]",
                cell_name, &expected_inputs[0]
            ),
            _ => format!(
                "There should be {} {}s in inputs{:?}",
                expected_inputs.len(),
                cell_name,
                expected_inputs
            ),
        }
    );

    das_assert!(
        current_outputs == expected_outputs,
        Error::InvalidTransactionStructure,
        "{}",
        match expected_outputs.len() {
            0 => format!("There should be none {} in outputs.", cell_name),
            1 => format!(
                "There should be only one {} in outputs[{}]",
                cell_name, &expected_outputs[0]
            ),
            _ => format!(
                "There should be {} {}s in outputs{:?}",
                expected_outputs.len(),
                cell_name,
                expected_outputs
            ),
        }
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
        // when the capacity is decreased, we need to make sure the capacity is bigger than basic_capacity
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
            input_capacity <= expected_fee + output_capacity, //  output_capacity >= input_capacity - expected_fee,
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

pub fn verify_das_get_change(expected_change: u64) -> Result<(), Error> {
    let das_wallet_lock = das_wallet_lock();
    let das_wallet_cells = util::find_cells_by_script(ScriptType::Lock, das_wallet_lock.as_reader(), Source::Output)?;

    let mut total_capacity = 0;
    for i in das_wallet_cells {
        let type_hash = high_level::load_cell_type_hash(i, Source::Output)?;
        das_assert!(
            type_hash.is_none(),
            Error::InvalidTransactionStructure,
            "outputs[{}] The cells to DAS should not contains any type script.",
            i
        );

        let capacity = high_level::load_cell_capacity(i, Source::Output)?;
        total_capacity += capacity;
    }

    das_assert!(
        total_capacity == expected_change,
        Error::ChangeError,
        "The change to DAS should be {} shannon, but {} found.",
        expected_change,
        total_capacity
    );

    Ok(())
}
