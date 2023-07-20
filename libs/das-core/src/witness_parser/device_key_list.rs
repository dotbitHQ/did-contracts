use core::slice::SlicePattern;

use alloc::collections::BTreeMap;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_cell_lock, load_cell_type_hash, QueryIter};
use molecule::bytes::Bytes;

use crate::constants::device_key_list_cell_type;

pub fn get_device_key_list_cell_deps() -> BTreeMap<[u8; 32], ([u8; 32], Bytes)> {
    let cell_deps = QueryIter::new(load_cell_type_hash, Source::CellDep)
        .filter_map(|hash| {
            hash.and_then(|h| {
                if h == device_key_list_cell_type().code_hash().raw_data().as_slice() {
                    Some(h)
                } else {
                    None
                }
            })
        })
        .flat_map(|hash| {
            QueryIter::new(
                move |index, source| {
                    let lock = load_cell_lock(index, source)?;
                    let mut buf: [u8; 32] = [0; 32];
                    let _ = ckb_std::syscalls::load_cell_data(&mut buf, 0, index, source);
                    Ok((buf, (hash, lock.args().raw_data())))
                },
                Source::CellDep,
            )
        })
        .collect();

    cell_deps
}
