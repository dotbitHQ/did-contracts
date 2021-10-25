use alloc::borrow::ToOwned;

pub fn get_owner_type(data: &[u8]) -> u8 {
    data.get(0)
        .expect("Das-lock should have some bytes for owner lock hash.")
        .to_owned()
}

pub fn get_owner_lock_args(data: &[u8]) -> &[u8] {
    if data[0] == 1 {
        data.get(1..29)
            .expect("Das-lock should have some bytes for owner lock hash.")
    } else {
        data.get(1..21)
            .expect("Das-lock should have some bytes for owner lock hash.")
    }
}

pub fn get_manager_type(data: &[u8]) -> u8 {
    if data[0] == 1 {
        data.get(29)
            .expect("Das-lock should have some bytes for manager lock hash.")
            .to_owned()
    } else {
        data.get(21)
            .expect("Das-lock should have some bytes for manager lock hash.")
            .to_owned()
    }
}

pub fn get_manager_lock_args(data: &[u8]) -> &[u8] {
    if data[0] == 1 {
        // skip 1 byte of manager lock type, so it is 29 + 1
        data.get(30..)
            .expect("Das-lock should have some bytes for manager lock hash.")
    } else {
        // skip 1 byte of manager lock type, so it is 21 + 1
        data.get(22..)
            .expect("Das-lock should have some bytes for manager lock hash.")
    }
}
