pub fn get_owner_lock_args(data: &[u8]) -> &[u8] {
    if data[0] == 1 {
        data.get(1..29).unwrap()
    } else {
        data.get(1..21).unwrap()
    }
}

pub fn get_manager_lock_args(data: &[u8]) -> &[u8] {
    if data[0] == 1 {
        // skip 1 byte of manager lock type, so it is 29 + 1
        data.get(30..).unwrap()
    } else {
        // skip 1 byte of manager lock type, so it is 21 + 1
        data.get(22..).unwrap()
    }
}
