#![allow(unused_imports)]
#![allow(unused_variables)]

use alloc::boxed::Box;
use alloc::string::String;
use core::convert::TryInto;

use ckb_std::ckb_constants::Source;
use das_types::mixer::{AccountCellDataReaderMixer, PreAccountCellDataReaderMixer};
use das_types::packed::*;
use das_types::prelude::*;
use das_types::prettier::Prettier;
use das_types::util::hex_string;

use super::data_parser::{account_cell, apply_register_cell, pre_account_cell};
use super::debug;

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
