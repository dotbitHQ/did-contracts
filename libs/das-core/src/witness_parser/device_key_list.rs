use alloc::collections::BTreeMap;
use core::slice::SlicePattern;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_cell_lock, load_cell_type_hash, QueryIter};
use molecule::bytes::Bytes;

use crate::constants::device_key_list_cell_type;

pub fn get_device_key_list_cell_deps() -> BTreeMap<[u8; 32], Bytes> {
    let cell_deps = QueryIter::new(
        |index, source| {
            let res = load_cell_type_hash(index, source)?;
            Ok(res.map(|hash| (index, hash)))
        },
        Source::CellDep,
    )
    .filter_map(|res| {
        res.and_then(|(index, hash)| {
            if hash == device_key_list_cell_type().code_hash().raw_data().as_slice() {
                Some((index, hash))
            } else {
                None
            }
        })
    })
    .map(|(index, _hash)| {
        let lock = load_cell_lock(index, Source::CellDep).unwrap();
        let mut buf: [u8; 32] = [0; 32];
        let _ = ckb_std::syscalls::load_cell_data(&mut buf, 0, index, Source::CellDep);

        (buf, lock.args().raw_data())
    })
    .collect();

    cell_deps
}
