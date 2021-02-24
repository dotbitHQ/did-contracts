use alloc::vec::Vec;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(32..52).unwrap()
}

pub fn get_next(data: &Vec<u8>) -> &[u8] {
    data.get(52..72).unwrap()
}

pub fn get_registered_at(data: &Vec<u8>) -> &[u8] {
    data.get(72..80).unwrap()
}

pub fn get_expired_at(data: &Vec<u8>) -> &[u8] {
    data.get(80..88).unwrap()
}

pub fn get_account(data: &Vec<u8>) -> &[u8] {
    data.get(88..).unwrap()
}
