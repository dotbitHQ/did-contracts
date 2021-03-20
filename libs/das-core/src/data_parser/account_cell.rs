use alloc::vec::Vec;
use core::convert::TryInto;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(32..42).unwrap()
}

pub fn get_next(data: &Vec<u8>) -> &[u8] {
    data.get(42..52).unwrap()
}

pub fn get_expired_at(data: impl AsRef<[u8]>) -> u64 {
    let bytes = data.as_ref().get(52..60).unwrap();
    u64::from_le_bytes(bytes.try_into().unwrap())
}

pub fn get_account(data: &Vec<u8>) -> &[u8] {
    data.get(60..).unwrap()
}
