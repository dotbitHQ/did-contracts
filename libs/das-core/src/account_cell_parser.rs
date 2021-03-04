use alloc::vec::Vec;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(32..42).unwrap()
}

pub fn get_next(data: &Vec<u8>) -> &[u8] {
    data.get(42..52).unwrap()
}

pub fn get_expired_at(data: &Vec<u8>) -> &[u8] {
    data.get(52..60).unwrap()
}

pub fn get_account(data: &Vec<u8>) -> &[u8] {
    data.get(60..).unwrap()
}
