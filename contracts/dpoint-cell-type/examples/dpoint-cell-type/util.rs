use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level;
use das_core::error::ScriptError;
use das_core::{code_to_error, data_parser, warn};
use dpoint_cell_type::error::ErrorCode;

pub fn get_total_dpoint(indexes: &[usize], source: Source) -> Result<u64, Box<dyn ScriptError>> {
    let mut total = 0;
    for i in indexes.iter() {
        let data = high_level::load_cell_data(*i, source)?;
        let dp = match data_parser::dpoint_cell::get_value(&data) {
            Some(dp) => dp,
            None => {
                warn!("{:?}[{}] The data of DPointCell is corrupted.", source, i);
                return Err(code_to_error!(ErrorCode::TheValueFieldIsMissing));
            }
        };
        total += dp;
    }

    Ok(total)
}

pub fn group_cells_by_lock(
    indexes: &[usize],
    source: Source,
) -> Result<BTreeMap<[u8; 32], Vec<usize>>, Box<dyn ScriptError>> {
    let mut group: BTreeMap<[u8; 32], Vec<usize>> = BTreeMap::new();
    for i in indexes.iter() {
        let lock_hash = high_level::load_cell_lock_hash(*i, source)?;
        if !group.contains_key(&lock_hash) {
            group.insert(lock_hash, vec![*i]);
        } else {
            let cells = group.get_mut(&lock_hash).unwrap();
            cells.push(*i);
        }
    }

    Ok(group)
}
