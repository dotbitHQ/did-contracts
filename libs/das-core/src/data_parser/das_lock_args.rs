use alloc::borrow::ToOwned;

pub fn get_owner_type(data: &[u8]) -> u8 {
    data.get(0)
        .expect("Das-lock should have some bytes for owner lock hash.")
        .to_owned()
}

pub fn get_owner_lock_args(data: &[u8]) -> &[u8] {
    let ret = match data[0] {
        1 => data.get(1..29),
        6 => data.get(1..33),
        _ => data.get(1..21),
    };

    ret.expect("Das-lock should have some bytes for owner lock hash.")
}

pub fn get_manager_type(data: &[u8]) -> u8 {
    let ret = match data[0] {
        1 => data.get(29),
        6 => data.get(33),
        _ => data.get(21),
    };

    ret.expect("Das-lock should have some bytes for manager lock hash.")
        .to_owned()
}

pub fn get_manager_lock_args(data: &[u8]) -> &[u8] {
    let ret = match data[0] {
        1 => data.get(30..),
        6 => data.get(34..),
        _ => data.get(22..),
    };

    ret.expect("Das-lock should have some bytes for manager lock hash.")
}
