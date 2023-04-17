use core::convert::TryInto;

use das_types::constants::{SubAccountConfigFlag, SubAccountCustomRuleFlag};

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

pub fn get_flag(data: &[u8]) -> Option<SubAccountConfigFlag> {
    match data.get(48) {
        Some(v) => SubAccountConfigFlag::try_from(*v).ok(),
        None => None,
    }
}

pub fn get_custom_script(data: &[u8]) -> Option<&[u8]> {
    // Compare with `data.get(48..81)`, This will allow getting custom_script which length is less than 32 bytes.
    data.get(49..).map(|v| if v.len() > 32 { &v[..32] } else { v })
}

pub fn get_custom_script_args(data: &[u8]) -> Option<&[u8]> {
    data.get(81..)
}

pub fn get_custom_rule_status_flag(data: &[u8]) -> Option<SubAccountCustomRuleFlag> {
    match data.get(49) {
        Some(v) => SubAccountCustomRuleFlag::try_from(*v).ok(),
        None => None,
    }
}

pub fn get_price_rules_hash(data: &[u8]) -> Option<&[u8]> {
    data.get(50..60)
}

pub fn get_preserved_rules_hash(data: &[u8]) -> Option<&[u8]> {
    data.get(60..70)
}
