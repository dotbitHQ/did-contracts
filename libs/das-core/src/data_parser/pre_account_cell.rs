use alloc::vec::Vec;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(32..).unwrap()
}
