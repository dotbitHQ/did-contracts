use crate::constants::ACCOUNT_ID_LENGTH;
use alloc::vec::Vec;
use core::convert::TryInto;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(32..(32 + ACCOUNT_ID_LENGTH))
        .expect("AccountCell should have at least 80 bytes of data.")
}

pub fn get_next(data: &Vec<u8>) -> &[u8] {
    let start = 32 + ACCOUNT_ID_LENGTH;
    data.get(start..(start + ACCOUNT_ID_LENGTH))
        .expect("AccountCell should have at least 80 bytes of data.")
}

pub fn get_expired_at(data: impl AsRef<[u8]>) -> u64 {
    let start = 32 + ACCOUNT_ID_LENGTH * 2;
    let bytes = data
        .as_ref()
        .get(start..(start + 8))
        .expect("AccountCell should have at least 80 bytes of data.");
    u64::from_le_bytes(bytes.try_into().unwrap())
}

pub fn get_account(data: &Vec<u8>) -> &[u8] {
    let start = 32 + ACCOUNT_ID_LENGTH * 2 + 8;
    data.get(start..)
        .expect("AccountCell should have some bytes after the leading 80 bytes for account.")
}
