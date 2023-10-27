pub fn get_value(data: &[u8]) -> Option<u64> {
    data.get(4..12)
        .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
        .or(Some(0))
}
