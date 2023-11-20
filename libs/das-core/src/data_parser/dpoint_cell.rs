pub fn get_value(data: &[u8]) -> Option<u64> {
    if data.len() > 12 {
        return None;
    }

    let header = match data.get(0..4) {
        Some(bytes) => u32::from_le_bytes(bytes.try_into().unwrap()) as usize,
        None => return None,
    };

    if header != 8 {
        return None;
    }

    match data.get(4..(4 + header)) {
        Some(bytes) => Some(u64::from_le_bytes(bytes.try_into().unwrap())),
        None => None,
    }
}
