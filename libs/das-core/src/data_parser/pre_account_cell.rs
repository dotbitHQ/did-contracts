pub fn get_id(data: &[u8]) -> &[u8] {
    data.get(32..).expect("PreAccountCell should have 40 bytes of data.")
}
