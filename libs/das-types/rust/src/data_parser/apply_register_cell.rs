#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "no_std")]
use core::convert::TryInto;
#[cfg(not(feature = "no_std"))]
use std::convert::TryInto;

pub fn get_account_hash(data: &[u8]) -> Option<Vec<u8>> {
    data.get(..32).map(|bytes| bytes.to_vec())
}

pub fn get_height(data: &[u8]) -> Option<u64> {
    data.get(32..40).map(|v| u64::from_le_bytes(v.try_into().unwrap()))
}

pub fn get_timestamp(data: &[u8]) -> Option<u64> {
    data.get(40..48).map(|v| u64::from_le_bytes(v.try_into().unwrap()))
}
