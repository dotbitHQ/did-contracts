#[cfg(feature = "no_std")]
use core::convert::TryInto;
#[cfg(not(feature = "no_std"))]
use std::convert::TryInto;

use super::super::constants::ACCOUNT_ID_LENGTH;

pub fn get_id(data: &[u8]) -> Option<&[u8]> {
    data.get(32..(32 + ACCOUNT_ID_LENGTH))
}

pub fn get_next(data: &[u8]) -> Option<&[u8]> {
    let start = 32 + ACCOUNT_ID_LENGTH;
    data.get(start..(start + ACCOUNT_ID_LENGTH))
}

pub fn get_expired_at(data: &[u8]) -> Option<u64> {
    let start = 32 + ACCOUNT_ID_LENGTH * 2;

    data.get(start..(start + 8))
        .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
}

pub fn get_account(data: &[u8]) -> Option<&[u8]> {
    let start = 32 + ACCOUNT_ID_LENGTH * 2 + 8;
    data.get(start..)
}
