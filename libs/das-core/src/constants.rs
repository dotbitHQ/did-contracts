use super::types::ScriptLiteral;
use super::util;
use alloc::{vec, vec::Vec};
use ckb_std::ckb_types::packed::*;

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

pub const ACCOUNT_CELL_BASIC_CAPACITY: u64 = 16_200_000_000;
pub const REF_CELL_BASIC_CAPACITY: u64 = 11_400_000_000;
pub const WALLET_CELL_BASIC_CAPACITY: u64 = 9_400_000_000;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_SUFFIX: &str = ".bit";

pub const DAS_WALLET_ID: [u8; 20] = [
    38, 188, 217, 147, 236, 105, 146, 36, 129, 210, 111, 101, 208, 215, 120, 89, 37, 48, 222, 94,
];

pub fn super_lock() -> Script {
    #[cfg(debug_assertions)]
    let super_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(not(debug_assertions))]
    let super_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            94, 176, 12, 14, 81, 175, 181, 55, 252, 128, 113, 129, 0, 52, 206, 146, 249, 140, 50,
            89,
        ],
    };

    util::script_literal_to_script(super_lock)
}

pub fn oracle_lock() -> Script {
    #[cfg(debug_assertions)]
    let oracle_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(not(debug_assertions))]
    let oracle_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    util::script_literal_to_script(oracle_lock)
}

#[cfg(debug_assertions)]
pub const TIME_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(not(debug_assertions))]
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

#[cfg(debug_assertions)]
pub const ALWAYS_SUCCESS_LOCK: ScriptLiteral = ScriptLiteral {
    code_hash: [
        157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224, 98,
        190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(not(debug_assertions))]
pub const ALWAYS_SUCCESS_LOCK: ScriptLiteral = ScriptLiteral {
    code_hash: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};
