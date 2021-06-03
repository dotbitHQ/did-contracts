#![allow(unused_imports)]
#![allow(unused_variables)]

use super::data_parser::{account_cell, pre_account_cell};
use super::debug;
use ckb_std::ckb_constants::Source;
use core::convert::TryInto;
use das_types::{packed::*, prelude::*, util::hex_string};

pub fn apply_register_cell(source: Source, index: usize, data: &Vec<u8>) {
    debug!(
        "  {:?}[{}].data: {{ hash: 0x{}, height: {} }}",
        source,
        index,
        hex_string(data.get(..32).unwrap()),
        u64::from_le_bytes(data.get(32..).unwrap().try_into().unwrap())
    );
}

pub fn pre_account_cell(source: Source, index: usize, data: &Vec<u8>, witness: Bytes) {
    debug!(
        "  {:?}[{}].data: {{ id: 0x{} }}",
        source,
        index,
        hex_string(pre_account_cell::get_id(&data))
    );
    let witness_data = PreAccountCellData::new_unchecked(witness.raw_data());
    debug!("  {:?}[{}].witness: {}", source, index, witness_data);
}

pub fn account_cell(source: Source, index: usize, data: &Vec<u8>, witness: Bytes) {
    debug!(
        "  {:?}[{}].data: {{ id: 0x{}, next: 0x{}, expired_at: {}, account: 0x{} }}",
        source,
        index,
        hex_string(account_cell::get_id(&data)),
        hex_string(account_cell::get_next(&data)),
        account_cell::get_expired_at(&data),
        hex_string(account_cell::get_account(&data))
    );
    let witness_data = AccountCellData::new_unchecked(witness.raw_data());
    debug!("  {:?}[{}].witness: {}", source, index, witness_data);
}
