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
    // Compare with `data.get(48..81)`, This will allow getting custom_script which length is less than 33 bytes.
    data.get(48..)
        .map(|v| if v.len() > 33 { &v[..33] } else { v })
        .or(Some(&[]))
}

pub fn get_custom_script_args(data: &[u8]) -> Option<&[u8]> {
    data.get(81..).or(Some(&[]))
}
