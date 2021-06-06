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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Debug)]
pub enum TypeScript {
    AccountCellType,
    ApplyRegisterCellType,
    BiddingCellType,
    IncomeCellType,
    OnSaleCellType,
    PreAccountCellType,
    ProposalCellType,
}

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const CELL_BASIC_CAPACITY: u64 = 6_100_000_000;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_ID_LENGTH: usize = 10;
pub const ACCOUNT_SUFFIX: &str = ".bit";
pub const ACCOUNT_MAX_PRICED_LENGTH: u8 = 8;

pub const PROPOSAL_CREATOR_WALLET_ID: [u8; ACCOUNT_ID_LENGTH] =
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 254];
pub const PROPOSAL_CONFIRMOR_WALLET_ID: [u8; ACCOUNT_ID_LENGTH] =
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 253];

pub fn super_lock() -> Script {
    #[cfg(feature = "dev")]
    let super_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "local")]
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

    #[cfg(feature = "testnet")]
    let super_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            188, 80, 42, 52, 164, 48, 227, 225, 103, 200, 42, 36, 219, 111, 146, 55, 177, 94, 191,
            53,
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
    #[cfg(feature = "dev")]
    let oracle_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

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
            236, 23, 170, 82, 25, 170, 33, 14, 115, 96, 92, 6, 197, 65, 72, 96, 224, 197, 232, 228,
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

pub fn das_wallet_lock() -> Script {
    #[cfg(feature = "dev")]
    let das_wallet_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "local")]
    let das_wallet_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    #[cfg(feature = "testnet")]
    let das_wallet_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            239, 191, 73, 127, 117, 47, 247, 166, 85, 168, 236, 111, 60, 143, 63, 234, 174, 214,
            228, 16,
        ],
    };

    #[cfg(feature = "mainnet")]
    let das_wallet_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
    };

    util::script_literal_to_script(das_wallet_lock)
}

pub fn das_lock() -> Script {
    #[cfg(feature = "dev")]
    let das_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "local")]
    let das_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            184, 243, 231, 77, 189, 72, 86, 149, 58, 151, 112, 104, 42, 255, 194, 137, 221, 0, 152,
            153, 45, 17, 214, 103, 205, 243, 84, 151, 226, 103, 190, 50,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "testnet")]
    let das_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            50, 109, 241, 102, 227, 240, 169, 0, 160, 174, 224, 67, 227, 26, 77, 234, 15, 1, 234,
            51, 7, 230, 226, 53, 240, 157, 27, 66, 32, 183, 95, 189,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "mainnet")]
    let das_lock: ScriptLiteral = ScriptLiteral {
        code_hash: [
            2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
            2, 2, 2,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    util::script_literal_to_script(das_lock)
}

pub fn time_cell_type() -> Script {
    #[cfg(feature = "dev")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

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
            215, 132, 35, 68, 147, 32, 41, 28, 65, 173, 204, 231, 65, 39, 108, 71, 223, 29, 187,
            11, 202, 33, 45, 0, 23, 219, 102, 41, 123, 232, 143, 25,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            36, 143, 0, 242, 165, 148, 174, 152, 37, 1, 17, 50, 103, 212, 135, 172, 210, 123, 52,
            62, 8, 30, 4, 209, 253, 4, 144, 179, 40, 139, 56, 217, 0, 0, 0, 0,
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
    #[cfg(feature = "dev")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

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
            122, 109, 182, 121, 62, 207, 52, 31, 143, 82, 137, 188, 22, 77, 74, 65, 124, 90, 219,
            153, 171, 134, 167, 80, 35, 13, 125, 20, 231, 55, 104, 231,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            81, 35, 192, 116, 254, 239, 16, 181, 140, 6, 27, 109, 22, 167, 10, 57, 123, 48, 149,
            112, 36, 242, 162, 98, 16, 34, 6, 33, 58, 128, 141, 55, 0, 0, 0, 0,
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

#[cfg(feature = "dev")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        8, 107, 220, 190, 240, 171, 98, 141, 49, 174, 209, 231, 186, 162, 100, 22, 211, 189, 225,
        226, 66, 165, 164, 125, 221, 174, 192, 110, 135, 229, 149, 208,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "local")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        228, 211, 239, 135, 78, 141, 98, 140, 101, 79, 184, 80, 81, 32, 235, 206, 205, 65, 87, 48,
        111, 174, 11, 234, 97, 164, 243, 23, 248, 121, 73, 202,
    ],
    hash_type: ScriptHashType::Type,
    args: Vec::new(),
};

#[cfg(feature = "testnet")]
pub const CONFIG_CELL_TYPE: ScriptLiteral = ScriptLiteral {
    code_hash: [
        3, 10, 194, 172, 217, 192, 22, 249, 164, 171, 19, 213, 44, 36, 77, 35, 170, 234, 99, 110,
        12, 189, 56, 110, 198, 96, 183, 153, 116, 148, 101, 23,
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

pub fn always_success_lock() -> Script {
    #[cfg(feature = "dev")]
    let always_success_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "local")]
    let always_success_lock = ScriptLiteral {
        code_hash: [
            184, 243, 231, 77, 189, 72, 86, 149, 58, 151, 112, 104, 42, 255, 194, 137, 221, 0, 152,
            153, 45, 17, 214, 103, 205, 243, 84, 151, 226, 103, 190, 50,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "testnet")]
    let always_success_lock = ScriptLiteral {
        code_hash: [
            241, 239, 97, 182, 151, 117, 8, 217, 236, 86, 254, 67, 57, 154, 1, 229, 118, 8, 106,
            118, 207, 15, 124, 104, 125, 20, 24, 51, 94, 140, 64, 31,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(feature = "mainnet")]
    let always_success_lock = ScriptLiteral {
        code_hash: [
            2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
            2, 2, 2,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    util::script_literal_to_script(always_success_lock)
}

pub fn signall_lock() -> Script {
    #[cfg(feature = "dev")]
    let signall_lock = ScriptLiteral {
        code_hash: [
            157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224,
            98, 190, 230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    #[cfg(not(feature = "dev"))]
    let signall_lock = ScriptLiteral {
        code_hash: [
            155, 215, 224, 111, 62, 207, 75, 224, 242, 252, 210, 24, 139, 35, 241, 185, 252, 200,
            142, 93, 75, 101, 168, 99, 123, 23, 114, 59, 189, 163, 204, 232,
        ],
        hash_type: ScriptHashType::Type,
        args: Vec::new(),
    };

    util::script_literal_to_script(signall_lock)
}
