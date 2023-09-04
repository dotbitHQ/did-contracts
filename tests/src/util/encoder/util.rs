pub fn length_of(data: &[u8]) -> Vec<u8> {
    (data.len() as u32).to_le_bytes().to_vec()
}
