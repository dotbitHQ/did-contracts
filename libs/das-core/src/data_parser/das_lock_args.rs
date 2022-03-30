use super::super::error::Error;
use alloc::borrow::ToOwned;

pub fn get_owner_type_opt(data: &[u8]) -> Option<u8> {
    data.get(0).map(|v| v.to_owned())
}

pub fn get_owner_type(data: &[u8]) -> u8 {
    get_owner_type_opt(data).expect("Das-lock should have some bytes for owner lock hash.")
}

pub fn get_owner_lock_args_opt(data: &[u8]) -> Option<&[u8]> {
    let ret = match data[0] {
        1 => data.get(1..29),
        6 => data.get(1..33),
        _ => data.get(1..21),
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
        _ => data.get(21),
    };

    ret.map(|v| v.to_owned())
}

pub fn get_manager_type(data: &[u8]) -> u8 {
    get_manager_type_opt(data).expect("Das-lock should have some bytes for manager lock hash.")
}

pub fn get_manager_lock_args_opt(data: &[u8]) -> Option<&[u8]> {
    let ret = match data[0] {
        1 => data.get(30..),
        6 => data.get(34..),
        _ => data.get(22..),
    };

    ret
}

pub fn get_manager_lock_args(data: &[u8]) -> &[u8] {
    get_manager_lock_args_opt(data).expect("Das-lock should have some bytes for manager lock hash.")
}

pub fn get_owner_and_manager(data: &[u8]) -> Result<(u8, &[u8], u8, &[u8]), Error> {
    let owner_type = get_owner_type_opt(data).ok_or(Error::DasLockArgsInvalid)?;
    let owner_args = get_owner_lock_args_opt(data).ok_or(Error::DasLockArgsInvalid)?;
    let manager_type = get_manager_type_opt(data).ok_or(Error::DasLockArgsInvalid)?;
    let manager_args = get_manager_lock_args_opt(data).ok_or(Error::DasLockArgsInvalid)?;

    Ok((owner_type, owner_args, manager_type, manager_args))
}
