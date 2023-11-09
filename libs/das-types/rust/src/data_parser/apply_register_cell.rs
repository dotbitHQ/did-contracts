use alloc::vec::Vec;
use core::convert::TryInto;

pub fn get_account_hash(data: &[u8]) -> Option<Vec<u8>> {
    data.get(..32).map(|bytes| bytes.to_vec())
}

pub fn get_height(data: &[u8]) -> u64 {
    let raw = data
        .get(32..40)
        .expect("ApplyRegisterCell should have 48 bytes of data.");
    u64::from_le_bytes(raw.try_into().unwrap())
}

pub fn get_timestamp(data: &[u8]) -> u64 {
    let raw = data
        .get(40..48)
        .expect("ApplyRegisterCell should have 48 bytes of data.");
    u64::from_le_bytes(raw.try_into().unwrap())
}
