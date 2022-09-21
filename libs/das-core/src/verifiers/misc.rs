use crate::{
    assert, code_to_error,
    constants::*,
    error::*,
    util::{self, find_cells_by_script},
    warn,
};
use alloc::boxed::Box;
use ckb_std::{ckb_constants::Source, ckb_types::packed as ckb_packed, high_level, syscalls::SysError};
use das_types::packed::*;

pub fn verify_no_more_cells(cells: &[usize], source: Source) -> Result<(), Box<dyn ScriptError>> {
    assert!(
        cells[0] == 0,
        ErrorCode::InvalidTransactionStructure,
        "{:?}[{}] The cell should be the first cell in {:?}.",
        source,
        cells[0],
        source
    );

    let last;
    if cells.len() == 1 {
        last = 0;
    } else if cells.len() > 1 {
        let mut prev_index = 0;
        for i in cells.iter().skip(1) {
            assert!(
                *i == prev_index + 1,
                ErrorCode::InvalidTransactionStructure,
                "{:?}[{}] The cell is missing in the verification array.",
                source,
                prev_index + 1
            );

            prev_index = *i
        }

        last = prev_index;
    } else {
        panic!("Can not verify empty array of cells.")
    }

    match high_level::load_cell_capacity(last + 1, source) {
        Err(SysError::IndexOutOfBound) => Ok(()), // This is Ok.
        _ => {
            warn!(
                "{:?}[{}] The cell should be the last cell in {:?}.",
                source, last, source
            );

            Err(code_to_error!(ErrorCode::InvalidTransactionStructure))
        }
    }
}

pub fn verify_no_more_cells_with_same_lock(
    lock: ckb_packed::ScriptReader,
    cells: &[usize],
    source: Source,
) -> Result<(), Box<dyn ScriptError>> {
    let cells_with_same_lock = find_cells_by_script(ScriptType::Lock, lock, source)?;

    for i in cells_with_same_lock {
        if !cells.contains(&i) {
            warn!(
                "{:?}[{}] There should be no more cells with the same lock.(lock_script: {})",
                source, i, lock
            );
            return Err(code_to_error!(ErrorCode::InvalidTransactionStructure));
        }
    }

    Ok(())
}

/// CAREFUL The codes below just support das-lock.
pub fn verify_user_get_change(
    config_main: ConfigCellMainReader,
    user_lock_reader: ckb_packed::ScriptReader,
    expected_output_balance: u64,
) -> Result<(), Box<dyn ScriptError>> {
    let output_balance_cells = util::find_balance_cells(config_main, user_lock_reader, Source::Output)?;
    let output_capacity = util::load_cells_capacity(&output_balance_cells, Source::Output)?;

    assert!(
        output_capacity >= expected_output_balance,
        ErrorCode::ChangeError,
        "The change should be {} shannon in outputs.(current: {}, user_lock: {})",
        expected_output_balance,
        output_capacity,
        util::hex_string(user_lock_reader.args().raw_data())
    );

    Ok(())
}

pub fn verify_user_get_change_when_inputs_removed(
    config_reader: ConfigCellMainReader,
    user_lock_reader: ckb_packed::ScriptReader,
    input_removed_cells: &[usize],
    output_created_cells: &[usize],
    extra_cost: u64,
) -> Result<(), Box<dyn ScriptError>> {
    let input_capacity = util::load_cells_capacity(input_removed_cells, Source::Input)?;
    let output_capacity = util::load_cells_capacity(output_created_cells, Source::Output)?;

    assert!(
        input_capacity >= output_capacity + extra_cost,
        ErrorCode::ChangeError,
        "The change to user is incorrectly input_capacity < output_capacity + extra_cost. (input: {}, output: {}, extra_cost: {})",
        input_capacity,
        output_capacity,
        extra_cost
    );

    verify_user_get_change(
        config_reader,
        user_lock_reader,
        input_capacity - output_capacity - extra_cost,
    )
}

pub fn verify_always_success_lock(index: usize, source: Source) -> Result<(), Box<dyn ScriptError>> {
    let lock = high_level::load_cell_lock(index, source).map_err(Error::from)?;
    let lock_reader = lock.as_reader();
    let always_success_lock = always_success_lock();
    let always_success_lock_reader = always_success_lock.as_reader();

    assert!(
        util::is_reader_eq(lock_reader.code_hash(), always_success_lock_reader.code_hash())
            && lock_reader.hash_type() == always_success_lock_reader.hash_type(),
        ErrorCode::AlwaysSuccessLockIsRequired,
        "The cell at {:?}[{}] should use always-success lock.(expected_code_hash: {})",
        source,
        index,
        always_success_lock.as_reader().code_hash()
    );

    Ok(())
}
