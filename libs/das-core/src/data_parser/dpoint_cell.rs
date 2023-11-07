pub fn get_value(data: &[u8]) -> Option<u64> {
    let header = match data.get(0..4) {
        Some(bytes) => u32::from_le_bytes(bytes.try_into().unwrap()),
        None => return None,
    };

    if header != 8 {
        return None;
    }

    match data.get(4..12) {
        Some(bytes) => Some(u64::from_le_bytes(bytes.try_into().unwrap())),
        None => None,
    }
}
