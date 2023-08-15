use alloc::borrow::ToOwned;
use alloc::boxed::Box;

use das_types::constants::DasLockType;

use super::super::error::*;

pub fn get_owner_type_opt(data: &[u8]) -> Option<u8> {
    data.get(0).map(|v| v.to_owned())
}

pub fn get_owner_type(data: &[u8]) -> u8 {
    get_owner_type_opt(data).expect("Das-lock should have some bytes for owner lock hash.")
}

pub fn get_owner_lock_args_opt(data: &[u8]) -> Option<&[u8]> {
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

pub fn get_owner_lock_args(data: &[u8]) -> &[u8] {
    get_owner_lock_args_opt(data).expect("Das-lock should have some bytes for owner lock hash.")
}

pub fn get_manager_type_opt(data: &[u8]) -> Option<u8> {
    let ret = match data[0] {
        1 => data.get(29),
        6 => data.get(33),
        8 => data.get(22),
        2 | 3 | 4 | 5 | 7 => data.get(21),
        _ => None,
    };

    ret.map(|v| v.to_owned())
}

pub fn get_manager_type(data: &[u8]) -> u8 {
    get_manager_type_opt(data).expect("Das-lock should have some bytes for manager lock hash.")
}

pub fn get_manager_lock_args_opt(data: &[u8]) -> Option<&[u8]> {
    // Validate if the algorithm id of manager is valid
    match get_manager_type_opt(data) {
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

pub fn get_manager_lock_args(data: &[u8]) -> &[u8] {
    get_manager_lock_args_opt(data).expect("Das-lock should have some bytes for manager lock hash.")
}

pub fn get_owner_and_manager(data: &[u8]) -> Result<(u8, &[u8], u8, &[u8]), Box<dyn ScriptError>> {
    let owner_type = get_owner_type_opt(data).ok_or(ErrorCode::DasLockArgsInvalid)?;
    let owner_args = get_owner_lock_args_opt(data).ok_or(ErrorCode::DasLockArgsInvalid)?;
    let manager_type = get_manager_type_opt(data).ok_or(ErrorCode::DasLockArgsInvalid)?;
    let manager_args = get_manager_lock_args_opt(data).ok_or(ErrorCode::DasLockArgsInvalid)?;

    Ok((owner_type, owner_args, manager_type, manager_args))
}
