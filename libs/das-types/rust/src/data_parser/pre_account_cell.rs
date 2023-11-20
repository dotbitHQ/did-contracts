pub fn get_id(data: &[u8]) -> Option<&[u8]> {
    data.get(32..)
}
