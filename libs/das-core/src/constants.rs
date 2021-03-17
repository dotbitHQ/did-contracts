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

#[derive(Debug)]
pub enum TypeScript {
    AccountCellType,
    ApplyRegisterCellType,
    PreAccountCellType,
    ProposalCellType,
    RefCellType,
    WalletCellType,
}

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const ACCOUNT_CELL_BASIC_CAPACITY: u64 = 14_600_000_000;
pub const REF_CELL_BASIC_CAPACITY: u64 = 10_500_000_000;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_ID_LENGTH: usize = 10;
pub const ACCOUNT_SUFFIX: &str = ".bit";
pub const ACCOUNT_MAX_PRICED_LENGTH: u8 = 8;

pub const BLOOM_FILTER_M: u64 = 1918;
pub const BLOOM_FILTER_K: u64 = 14;

pub const DAS_WALLET_ID: [u8; ACCOUNT_ID_LENGTH] = [183, 82, 104, 3, 246, 126, 190, 112, 171, 166];

pub fn super_lock() -> Script {
    #[cfg(feature = "local")]
    let super_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "testnet")]
    let super_lock = ScriptLiteral {
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

    #[cfg(feature = "mainnet")]
    let super_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
    };

    util::script_literal_to_script(super_lock)
}

pub fn oracle_lock() -> Script {
    #[cfg(feature = "local")]
    let oracle_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "testnet")]
    let oracle_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            199, 95, 213, 248, 173, 210, 160, 77, 185, 255, 202, 248, 139, 67, 125, 118, 241, 129,
            39, 151,
        ],
    };

    #[cfg(feature = "mainnet")]
    let oracle_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
    };

    util::script_literal_to_script(oracle_lock)
}

pub fn wallet_maker_lock() -> Script {
    #[cfg(feature = "local")]
    let wallet_maker_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "testnet")]
    let wallet_maker_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            231, 14, 55, 173, 211, 245, 169, 210, 67, 251, 214, 88, 159, 92, 49, 124, 73, 4, 141,
            202,
        ],
    };

    #[cfg(feature = "mainnet")]
    let wallet_maker_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
    };

    util::script_literal_to_script(wallet_maker_lock)
}

pub fn time_cell_type() -> Script {
    #[cfg(feature = "local")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "testnet")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            228, 253, 111, 70, 171, 31, 211, 213, 179, 119, 223, 158, 45, 78, 167, 126, 59, 82,
            245, 58, 195, 49, 149, 149, 187, 56, 208, 151, 234, 5, 28, 253,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            208, 193, 199, 21, 111, 46, 49, 10, 18, 130, 46, 44, 195, 54, 57, 142, 196, 239, 25,
            74, 188, 31, 150, 2, 59, 116, 63, 50, 73, 240, 158, 33, 2, 0, 0, 0,
        ],
    };

    #[cfg(feature = "mainnet")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            184, 243, 231, 77, 189, 72, 86, 149, 58, 151, 112, 104, 42, 255, 194, 137, 221, 0, 152,
            153, 45, 17, 214, 103, 205, 243, 84, 151, 226, 103, 190, 50,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![116, 105, 109, 101],
    };

    util::script_literal_to_script(time_cell_type)
}

pub fn height_cell_type() -> Script {
    #[cfg(feature = "local")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "testnet")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            95, 106, 76, 194, 205, 99, 105, 219, 207, 56, 221, 251, 196, 50, 60, 244, 105, 92, 46,
            140, 32, 174, 213, 114, 181, 219, 106, 220, 47, 175, 157, 80,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            225, 169, 88, 164, 193, 18, 175, 149, 161, 34, 12, 111, 238, 95, 150, 153, 114, 163,
            216, 206, 19, 251, 123, 50, 17, 247, 26, 187, 93, 177, 130, 65, 2, 0, 0, 0,
        ],
    };

    #[cfg(feature = "mainnet")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            184, 243, 231, 77, 189, 72, 86, 149, 58, 151, 112, 104, 42, 255, 194, 137, 221, 0, 152,
            153, 45, 17, 214, 103, 205, 243, 84, 151, 226, 103, 190, 50,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![104, 101, 105, 103, 104, 116],
    };

    util::script_literal_to_script(height_cell_type)
}

#[cfg(feature = "local")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        8, 107, 220, 190, 240, 171, 98, 141, 49, 174, 209, 231, 186, 162, 100, 22, 211, 189, 225,
        226, 66, 165, 164, 125, 221, 174, 192, 110, 135, 229, 149, 208,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "testnet")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        72, 159, 242, 25, 94, 212, 26, 172, 154, 146, 101, 198, 83, 216, 202, 87, 200, 37, 178, 45,
        183, 101, 185, 224, 141, 83, 117, 114, 255, 44, 188, 27,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "mainnet")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2,
        2, 2,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "local")]
pub const ALWAYS_SUCCESS_LOCK: ScriptLiteral = ScriptLiteral {
    code_hash: [
        157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224, 98,
        190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "testnet")]
pub const ALWAYS_SUCCESS_LOCK: ScriptLiteral = ScriptLiteral {
    code_hash: [
        241, 239, 97, 182, 151, 117, 8, 217, 236, 86, 254, 67, 57, 154, 1, 229, 118, 8, 106, 118,
        207, 15, 124, 104, 125, 20, 24, 51, 94, 140, 64, 31,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "mainnet")]
pub const ALWAYS_SUCCESS_LOCK: ScriptLiteral = ScriptLiteral {
    code_hash: [
        2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2,
        2, 2,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};
