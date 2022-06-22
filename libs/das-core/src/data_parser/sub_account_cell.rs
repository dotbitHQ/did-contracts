use core::convert::TryInto;

pub fn get_smt_root(data: &[u8]) -> Option<&[u8]> {
    data.get(..32)
}

pub fn get_das_profit(data: &[u8]) -> Option<u64> {
    data.get(32..40)
        .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
        .or(Some(0))
}

pub fn get_owner_profit(data: &[u8]) -> Option<u64> {
    data.get(40..48)
        .map(|v| u64::from_le_bytes(v.try_into().unwrap()))
        .or(Some(0))
}

pub fn get_custom_script(data: &[u8]) -> Option<&[u8]> {
    data.get(48..81)
}
