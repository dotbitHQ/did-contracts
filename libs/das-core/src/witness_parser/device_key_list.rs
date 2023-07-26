use alloc::collections::BTreeMap;
use core::slice::SlicePattern;

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{load_cell_lock, load_cell_type_hash, QueryIter};
use ckb_std::syscalls::SysError;
use molecule::bytes::Bytes;

use crate::constants::device_key_list_cell_type;

pub fn get_device_key_list_cell_deps() -> BTreeMap<[u8; 32], Bytes> {
    // let mut i = 0;
    // let mut cell_deps: BTreeMap<[u8; 32], Bytes> = Default::default();

    // loop {
    //     let type_hash = match load_cell_type_hash(i, Source::CellDep) {
    //         Ok(res) => res,
    //         Err(SysError::IndexOutOfBound) => break,
    //         e => e.unwrap(),
    //     };

    //     if let Some(type_hash) = type_hash {
    //         if type_hash == device_key_list_cell_type().code_hash().raw_data().as_slice() {
    //             let type_hash = load_cell_type_hash(i, Source::CellDep).unwrap().unwrap();
    //             debug!("type_hash: {:?}", type_hash);
    //             let lock = load_cell_lock(i, Source::CellDep).unwrap();
    //             let mut buf = [0; 32];
    //             let _ = ckb_std::syscalls::load_cell_data(&mut buf, 0, i, Source::CellDep);
    //             debug!("buf: {:?}", buf);
    //             debug!("lock args: {:?}", lock.args().raw_data());
    //             cell_deps.insert(buf, lock.args().raw_data());
    //         }
    //     }
        
    //     i += 1;
    // }
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
                let lock = load_cell_lock(index, Source::CellDep).unwrap();
                let mut buf: [u8; 32] = [0; 32];
                let _ = ckb_std::syscalls::load_cell_data(&mut buf, 0, index, Source::CellDep);
                Some((buf, lock.args().raw_data()))
            } else {
                None
            }
        })
    })
    // .map(|(index, _hash)| {
    //     let lock = load_cell_lock(index + 1, Source::CellDep).unwrap();
    //     let mut buf: [u8; 32] = [0; 32];
    //     let _ = ckb_std::syscalls::load_cell_data(&mut buf, 0, index + 1, Source::CellDep);
    //     debug!("index: {}", index + 1);
    //     debug!("buf: {:?}", buf);
    //     debug!("lock_arg: {:?}", lock.args().raw_data());
    //     (buf, lock.args().raw_data())
    // })
    .collect();

    cell_deps
}
