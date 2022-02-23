#![allow(unused_imports)]
#![allow(unused_variables)]

use super::{
    data_parser::{account_cell, apply_register_cell, pre_account_cell},
    debug,
};
use alloc::{boxed::Box, string::String};
use ckb_std::ckb_constants::Source;
use core::convert::TryInto;
use das_types::mixer::AccountCellDataReaderMixer;
use das_types::{packed::*, prelude::*, prettier::Prettier, util::hex_string};

#[cfg(debug_assertions)]
pub fn apply_register_cell(source: Source, index: usize, data: &Vec<u8>) {
    debug!("  ====== {:?}[{}] ApplyRegisterCell ↓ ======", source, index);

    debug!(
        "    data: {{ hash: 0x{}, height: {}, timestamp: {} }}",
        hex_string(data.get(..32).unwrap()),
        apply_register_cell::get_height(data),
        apply_register_cell::get_timestamp(data)
    );
}

#[cfg(debug_assertions)]
pub fn pre_account_cell(
    source: Source,
    index: usize,
    data: &Vec<u8>,
    raw_witness: Option<BytesReader>,
    witness_reader_opt: Option<PreAccountCellDataReader>,
) {
    debug!("  ====== {:?}[{}] PreAccountCell ↓ ======", source, index);

    debug!("    data: {{ id: 0x{} }}", hex_string(pre_account_cell::get_id(&data)));

    let witness_reader;
    if raw_witness.is_some() {
        witness_reader = PreAccountCellDataReader::new_unchecked(raw_witness.unwrap().raw_data());
    } else if witness_reader_opt.is_some() {
        witness_reader = witness_reader_opt.unwrap()
    } else {
        panic!("Must pass one of raw_witness and witness_reader_opt");
    }

    debug!("    witness: {}", witness_reader.as_prettier());
}

#[cfg(debug_assertions)]
pub fn account_cell<'r>(
    source: Source,
    index: usize,
    data: &Vec<u8>,
    version: u32,
    raw_witness: Option<BytesReader>,
    witness_reader_opt: Option<Box<dyn AccountCellDataReaderMixer + 'r>>,
) {
    debug!("  ====== {:?}[{}] AccountCell(v{}) ↓ ======", source, index, version);

    debug!(
        "    data: {{ hash: 0x{}, id: 0x{}, next: 0x{}, expired_at: {}, account: {} }}",
        hex_string(data.get(..32).unwrap()),
        hex_string(account_cell::get_id(&data)),
        hex_string(account_cell::get_next(&data)),
        account_cell::get_expired_at(&data),
        String::from_utf8(account_cell::get_account(&data).to_vec()).unwrap()
    );

    if raw_witness.is_some() {
        unreachable!();
    } else {
        let witness_reader = witness_reader_opt.expect("Must pass one of raw_witness and witness_reader_opt");
        if version == 2 {
            debug!("    witness: {}", witness_reader.try_into_v2().unwrap().as_prettier());
        } else {
            debug!(
                "    witness: {}",
                witness_reader.try_into_latest().unwrap().as_prettier()
            );
        }
    }
}

#[cfg(debug_assertions)]
pub fn income_cell(
    source: Source,
    index: usize,
    raw_witness: Option<BytesReader>,
    witness_reader_opt: Option<IncomeCellDataReader>,
) {
    debug!("  ====== {:?}[{}] IncomeCell ↓ ======", source, index);

    let witness_reader;
    if raw_witness.is_some() {
        witness_reader = IncomeCellDataReader::new_unchecked(raw_witness.unwrap().raw_data());
    } else if witness_reader_opt.is_some() {
        witness_reader = witness_reader_opt.unwrap();
    } else {
        panic!("Must pass one of raw_witness and witness_reader_opt");
    }

    debug!("    witness.creator: {}", witness_reader.creator());
    debug!("    witness.records: {} total", witness_reader.records().len());
    for (i, record) in witness_reader.records().iter().enumerate() {
        debug!(
            "      {{ index: {}, belong_to.args: {}, capacity: {} }}",
            i,
            record.belong_to().args(),
            u64::from(record.capacity())
        );
    }
}
