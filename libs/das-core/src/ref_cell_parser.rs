use alloc::vec::Vec;

pub fn get_id(data: &Vec<u8>) -> &[u8] {
    data.get(..10).unwrap()
}

pub fn get_is_owner(data: impl AsRef<[u8]>) -> bool {
    let byte = data.as_ref().get(10).unwrap();
    byte == &0u8
}
