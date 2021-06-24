#![allow(unused_imports)]
#![allow(unused_variables)]

use super::data_parser::{account_cell, pre_account_cell};
use super::debug;
use ckb_std::ckb_constants::Source;
use core::convert::TryInto;
use das_types::{packed::*, prelude::*, util::hex_string};

pub fn apply_register_cell(source: Source, index: usize, data: &Vec<u8>) {
    debug!(
        "  ====== {:?}[{}] ApplyRegisterCell ↓ ======",
        source, index
    );

    debug!(
        "    data: {{ hash: 0x{}, height: {} }}",
        hex_string(data.get(..32).unwrap()),
        u64::from_le_bytes(data.get(32..).unwrap().try_into().unwrap())
    );
}

pub fn pre_account_cell(
    source: Source,
    index: usize,
    data: &Vec<u8>,
    raw_witness: Option<BytesReader>,
    witness_reader_opt: Option<PreAccountCellDataReader>,
) {
    debug!("  ====== {:?}[{}] PreAccountCell ↓ ======", source, index);

    debug!(
        "    data: {{ id: 0x{} }}",
        hex_string(pre_account_cell::get_id(&data))
    );

    let witness_reader;
    if raw_witness.is_some() {
        witness_reader = PreAccountCellDataReader::new_unchecked(raw_witness.unwrap().raw_data());
    } else if witness_reader_opt.is_some() {
        witness_reader = witness_reader_opt.unwrap()
    } else {
        panic!("Must pass one of raw_witness and witness_reader_opt");
    }

    debug!("    witness: {}", witness_reader);
}

pub fn account_cell(
    source: Source,
    index: usize,
    data: &Vec<u8>,
    raw_witness: Option<BytesReader>,
    witness_reader_opt: Option<AccountCellDataReader>,
) {
    debug!("  ====== {:?}[{}] AccountCell ↓ ======", source, index);

    debug!(
        "    data: {{ id: 0x{}, next: 0x{}, expired_at: {}, account: 0x{} }}",
        hex_string(account_cell::get_id(&data)),
        hex_string(account_cell::get_next(&data)),
        account_cell::get_expired_at(&data),
        hex_string(account_cell::get_account(&data))
    );

    let witness_reader;
    if raw_witness.is_some() {
        witness_reader = AccountCellDataReader::new_unchecked(raw_witness.unwrap().raw_data());
    } else if witness_reader_opt.is_some() {
        witness_reader = witness_reader_opt.unwrap()
    } else {
        panic!("Must pass one of raw_witness and witness_reader_opt");
    }

    debug!("    witness: {}", witness_reader);
}

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
    debug!("    witness.records: ");
    for record in witness_reader.records().iter() {
        debug!(
            "      {{ belong_to.args: {}, capacity: {} }}",
            record.belong_to().args(),
            u64::from(record.capacity())
        );
    }
}
