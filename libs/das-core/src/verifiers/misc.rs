use crate::{assert, constants::*, error::Error, util, util::find_cells_by_script, warn};
use ckb_std::{ckb_constants::Source, ckb_types::packed as ckb_packed, high_level};
use das_types::packed::*;

pub fn verify_no_more_cells(cells: &[usize], source: Source) -> Result<(), Error> {
    assert!(
        cells[0] == 0,
        Error::InvalidTransactionStructure,
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
                Error::InvalidTransactionStructure,
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

    match high_level::load_cell_capacity(last + 1, source).map_err(Error::from) {
        Err(Error::IndexOutOfBound) => Ok(()), // This is Ok.
        _ => {
            warn!(
                "{:?}[{}] The cell should be the last cell in {:?}.",
                source, last, source
            );

            Err(Error::InvalidTransactionStructure)
        }
    }
}

pub fn verify_no_more_cells_with_same_lock(
    lock: ckb_packed::ScriptReader,
    cells: &[usize],
    source: Source,
) -> Result<(), Error> {
    let cells_with_same_lock = find_cells_by_script(ScriptType::Lock, lock, source)?;

    for i in cells_with_same_lock {
        if !cells.contains(&i) {
            return Err(Error::InvalidTransactionStructure);
        }
    }

    Ok(())
}

/// CAREFUL The codes below just support das-lock.
pub fn verify_user_get_change(
    config_main: ConfigCellMainReader,
    user_lock_reader: ckb_packed::ScriptReader,
    expected_balance: u64,
) -> Result<(), Error> {
    let mut current_capacity = 0;

    let balance_cells = util::find_balance_cells(config_main, user_lock_reader, Source::Output)?;
    for i in balance_cells {
        current_capacity += high_level::load_cell_capacity(i, Source::Output)?;
    }

    assert!(
        current_capacity >= expected_balance,
        Error::ChangeError,
        "The change should be {} shannon in outputs.(current: {}, user_lock: {})",
        expected_balance,
        current_capacity,
        user_lock_reader
    );

    Ok(())
}
