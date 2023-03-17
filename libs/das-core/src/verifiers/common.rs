use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use core::cmp::Ordering;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_types::packed;
use sparse_merkle_tree::ckb_smt::SMTBuilder;
use sparse_merkle_tree::H256;

use crate::constants::{das_wallet_lock, CellField, ScriptType};
use crate::error::*;
use crate::util;

pub fn verify_cell_dep_number(
    cell_name: &str,
    current_deps: &[usize],
    expected_deps_len: usize,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_deps.len() == expected_deps_len,
        ErrorCode::InvalidTransactionStructure,
        "{}",
        match expected_deps_len {
            0 => format!("There should be none {} in cell_deps.", cell_name),
            1 => format!("There should be only one {} in cell_deps.", cell_name),
            _ => format!("There should be {} {}s in cell_deps.", expected_deps_len, cell_name),
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
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_inputs.len() == expected_inputs_len,
        ErrorCode::InvalidTransactionStructure,
        "{}",
        match expected_inputs_len {
            0 => format!("There should be none {} in inputs.", cell_name),
            1 => format!("There should be only one {} in inputs.", cell_name),
            _ => format!("There should be {} {}s in inputs.", expected_inputs_len, cell_name),
        }
    );

    das_assert!(
        current_outputs.len() == expected_outputs_len,
        ErrorCode::InvalidTransactionStructure,
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
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if the number of {}s is correct.", cell_name);

    das_assert!(
        current_inputs.len().cmp(&expected_inputs_range.1) == expected_inputs_range.0,
        ErrorCode::InvalidTransactionStructure,
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
        ErrorCode::InvalidTransactionStructure,
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
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if the number and position of {}s is correct.", cell_name);

    das_assert!(
        current_inputs == expected_inputs,
        ErrorCode::InvalidTransactionStructure,
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
        ErrorCode::InvalidTransactionStructure,
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

/// WARNING! The witness will not be compared.
pub fn verify_cell_consistent_with_exception(
    cell_name: &str,
    input_cell_index: usize,
    output_cell_index: usize,
    except_fields: Vec<CellField>,
) -> Result<(), Box<dyn ScriptError>> {
    let input_cell = high_level::load_cell(input_cell_index, Source::Input).map_err(Error::<ErrorCode>::from)?;
    let output_cell = high_level::load_cell(output_cell_index, Source::Output).map_err(Error::<ErrorCode>::from)?;

    if !except_fields.contains(&CellField::Capacity) {
        debug!("Verify if the capacity of the {} is consistent ...", cell_name);

        let input_capacity = u64::from(packed::Uint64::from(input_cell.capacity()));
        let output_capacity = u64::from(packed::Uint64::from(output_cell.capacity()));

        das_assert!(
            input_capacity <= output_capacity,
            ErrorCode::CellCapacityMustBeConsistent,
            "The capacity of the {} should be consistent or increased.(input: {}, output: {})",
            cell_name,
            input_capacity,
            output_capacity
        );
    }

    if !except_fields.contains(&CellField::Lock) {
        debug!("Verify if the lock script of the {} is consistent ...", cell_name);

        let input_lock = input_cell.lock();
        let output_lock = output_cell.lock();

        das_assert!(
            util::is_entity_eq(&input_lock, &output_lock),
            ErrorCode::CellLockCanNotBeModified,
            "The lock of the {} should be consistent.(input: {}, output: {})",
            cell_name,
            input_lock,
            output_lock
        );
    }

    if !except_fields.contains(&CellField::Type) {
        debug!("Verify if the type script of the {} is consistent ...", cell_name);

        let input_type = input_cell.type_();
        let output_type = output_cell.type_();

        das_assert!(
            util::is_entity_eq(&input_type, &output_type),
            ErrorCode::CellLockCanNotBeModified,
            "The lock of the {} should be consistent.(input: {}, output: {})",
            cell_name,
            input_type,
            output_type
        );
    }

    if !except_fields.contains(&CellField::Data) {
        debug!("Verify if the data of the {} is consistent ...", cell_name);

        let input_data = util::load_cell_data(input_cell_index, Source::Input)?;
        let output_data = util::load_cell_data(output_cell_index, Source::Output)?;

        das_assert!(
            input_data == output_data,
            ErrorCode::CellLockCanNotBeModified,
            "The lock of the {} should be consistent.(input: {}, output: {})",
            cell_name,
            util::hex_string(&input_data),
            util::hex_string(&output_data)
        );
    }

    Ok(())
}

// The tx fee is from a specific cell (like AccountCell/AccountSaleCell), here to verify the validity of the fee spent
pub fn verify_tx_fee_spent_correctly(
    cell_name: &str,
    input_cell: usize,
    output_cell: usize,
    expected_fee: u64,
    basic_capacity: u64,
) -> Result<(), Box<dyn ScriptError>> {
    debug!("Verify if {} paid fee correctly.", cell_name);

    let input_capacity = high_level::load_cell_capacity(input_cell, Source::Input)?;
    let output_capacity = high_level::load_cell_capacity(output_cell, Source::Output)?;

    if input_capacity > output_capacity {
        // when the capacity is decreased, we need to make sure the capacity is bigger than basic_capacity
        assert!(
            output_capacity >= basic_capacity,
            ErrorCode::TxFeeSpentError, // changed from ErrorCode::AccountSaleCellFeeError
            "The {} has no more capacity as fee for this transaction.(input_capacity: {}, output_capacity: {}, basic_capacity: {})",
            cell_name,
            input_capacity,
            output_capacity,
            basic_capacity
        );

        assert!(
            input_capacity <= expected_fee + output_capacity, //  output_capacity >= input_capacity - expected_fee,
            ErrorCode::TxFeeSpentError,
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

pub fn verify_das_get_change(expected_change: u64) -> Result<(), Box<dyn ScriptError>> {
    let das_wallet_lock = das_wallet_lock();
    let das_wallet_cells = util::find_cells_by_script(ScriptType::Lock, das_wallet_lock.as_reader(), Source::Output)?;

    let mut total_capacity = 0;
    for i in das_wallet_cells {
        let type_hash = high_level::load_cell_type_hash(i, Source::Output)?;
        das_assert!(
            type_hash.is_none(),
            ErrorCode::InvalidTransactionStructure,
            "outputs[{}] The cells to DAS should not contains any type script.",
            i
        );

        let capacity = high_level::load_cell_capacity(i, Source::Output)?;
        total_capacity += capacity;
    }

    das_assert!(
        total_capacity == expected_change,
        ErrorCode::ChangeError,
        "The change to DAS should be {} shannon, but {} found.",
        expected_change,
        total_capacity
    );

    Ok(())
}

pub fn verify_smt_proof(
    key: [u8; 32],
    val: [u8; 32],
    root: [u8; 32],
    proof: &[u8],
) -> Result<(), Box<dyn ScriptError>> {
    let builder = SMTBuilder::new();
    let builder = builder.insert(&H256::from(key), &H256::from(val)).unwrap();

    let smt = builder.build().unwrap();
    let ret = smt.verify(&H256::from(root), &proof);
    if let Err(_e) = ret {
        debug!("  verify_smt_proof verification failed. Err: {:?}", _e);
        return Err(code_to_error!(ErrorCode::SMTProofVerifyFailed));
    } else {
        debug!("  verify_smt_proof verification passed.");
    }
    Ok(())
}
