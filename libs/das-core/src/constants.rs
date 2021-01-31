use super::types::ScriptLiteral;
use alloc::vec::Vec;

#[derive(Debug)]
#[repr(u8)]
pub enum ScriptHashType {
    Data = 0,
    Type = 1,
}

#[derive(Debug)]
pub enum ScriptType {
    Lock,
    Type,
}

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

// TODO Calculate real AccountCell base capacity
pub const ACCOUNT_CELL_BASE_CAPACITY: u64 = 200;

pub const TIME_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(debug_assertions)]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        8, 107, 220, 190, 240, 171, 98, 141, 49, 174, 209, 231, 186, 162, 100, 22, 211, 189, 225,
        226, 66, 165, 164, 125, 221, 174, 192, 110, 135, 229, 149, 208,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(not(debug_assertions))]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};
