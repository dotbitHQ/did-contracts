use crate::{assert, constants::*, error::Error, util, warn};
use ckb_std::{ckb_constants::Source, ckb_types::packed as ckb_packed, high_level};
use das_types::packed::ConfigCellMainReader;

pub fn verify_no_more_cells(cells: &[usize], source: Source) -> Result<(), Error> {
    assert!(
        cells[0] == 0,
        Error::InvalidTransactionStructure,
        "{:?}[{}] The cell should be the last cell in {:?}.",
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

/// CAREFUL The codes below just support das-lock.
pub fn verify_user_get_change(
    config_main: ConfigCellMainReader,
    user_lock_reader: ckb_packed::ScriptReader,
    expected_balance: u64,
) -> Result<(), Error> {
    let mut current_capacity = 0;

    let balance_cells = util::find_cells_by_type_id_and_filter(
        ScriptType::Type,
        config_main.type_id_table().balance_cell(),
        Source::Output,
        |i, source| {
            let lock = high_level::load_cell_lock(i, source)?;
            Ok(util::is_reader_eq(lock.as_reader(), user_lock_reader))
        },
    )?;

    for i in balance_cells {
        current_capacity += high_level::load_cell_capacity(i, Source::Output).map_err(Error::from)?;
    }

    assert!(
        current_capacity >= expected_balance,
        Error::ReverseRecordCellChangeError,
        "The change of the transaction should be {} shannon.(current: {})",
        expected_balance,
        current_capacity
    );

    Ok(())
}
