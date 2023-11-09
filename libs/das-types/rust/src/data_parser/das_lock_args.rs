#[cfg(feature = "no_std")]
use alloc::borrow::ToOwned;
#[cfg(feature = "no_std")]
use core::convert::TryFrom;
#[cfg(not(feature = "no_std"))]
use std::convert::TryFrom;

use super::super::constants::DasLockType;

pub fn get_owner_type(data: &[u8]) -> Option<u8> {
    data.get(0).map(|v| v.to_owned())
}

pub fn get_owner_lock_args(data: &[u8]) -> Option<&[u8]> {
    // TODO move the args length to a enum in das-types
    let ret = match data[0] {
        1 => data.get(1..29),
        6 => data.get(1..33),
        // TODO: temporary walkaround. WebAuthn has sub_alg_id. Currently we treat it as part of manager
        8 => data.get(1..22),
        2 | 3 | 4 | 5 | 7 => data.get(1..21),
        _ => None,
    };

    ret
}
pub fn get_manager_type(data: &[u8]) -> Option<u8> {
    let ret = match data[0] {
        1 => data.get(29),
        6 => data.get(33),
        8 => data.get(22),
        2 | 3 | 4 | 5 | 7 => data.get(21),
        _ => None,
    };

    ret.map(|v| v.to_owned())
}

pub fn get_manager_lock_args(data: &[u8]) -> Option<&[u8]> {
    // Validate if the algorithm id of manager is valid
    match get_manager_type(data) {
        Some(v) => {
            if DasLockType::try_from(v).is_err() {
                return None;
            }
        }
        _ => return None,
    };

    let ret = match data[0] {
        1 => data.get(30..),
        6 => data.get(34..),
        // TODO: temporary walkaround. WebAuthn has sub_alg_id. Currently we treat it as part of manager
        8 => data.get(23..),
        2 | 3 | 4 | 5 | 7 => data.get(22..),
        _ => None,
    };

    ret
}
