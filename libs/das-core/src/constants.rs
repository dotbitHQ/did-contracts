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

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OracleCellType {
    Quote = 0,
    Time = 1,
    Height = 2,
}

pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const CELL_BASIC_CAPACITY: u64 = 6_100_000_000;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_ID_LENGTH: usize = 20;
pub const ACCOUNT_SUFFIX: &str = ".bit";
pub const ACCOUNT_MAX_PRICED_LENGTH: u8 = 8;

pub const CUSTOM_KEYS_NAMESPACE: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz_";

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
            92, 80, 105, 235, 8, 87, 239, 198, 94, 27, 202, 12, 7, 223, 52, 195, 22, 99, 179, 98,
            47, 211, 135, 108, 135, 99, 32, 252, 150, 52, 226, 168,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![
            59, 73, 146, 224, 132, 95, 110, 120, 133, 181, 73, 146, 119, 250, 131, 105, 38, 136,
            164, 99,
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
49, 196, 64, 138, 2, 214, 213, 185, 252, 209, 202, 139, 84, 44, 8, 117, 92, 132, 166, 38, 94, 14, 1, 41, 224, 88, 10, 78, 144, 77, 65, 141
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
        args: vec![1],
    };

    #[cfg(feature = "local")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1],
    };

    #[cfg(feature = "testnet")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            150, 36, 140, 222, 251, 9, 238, 217, 16, 1, 138, 132, 124, 251, 81, 173, 4, 76, 45,
            125, 182, 80, 17, 41, 49, 118, 14, 62, 243, 74, 126, 154,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1],
    };

    #[cfg(feature = "mainnet")]
    let time_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![1],
    };

    util::script_literal_to_script(time_cell_type)
}

pub fn height_cell_type() -> Script {
    #[cfg(feature = "dev")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![2],
    };

    #[cfg(feature = "local")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![2],
    };

    #[cfg(feature = "testnet")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            150, 36, 140, 222, 251, 9, 238, 217, 16, 1, 138, 132, 124, 251, 81, 173, 4, 76, 45,
            125, 182, 80, 17, 41, 49, 118, 14, 62, 243, 74, 126, 154,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![2],
    };

    #[cfg(feature = "mainnet")]
    let height_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![2],
    };

    util::script_literal_to_script(height_cell_type)
}

pub fn quote_cell_type() -> Script {
    #[cfg(feature = "dev")]
    let quote_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0],
    };

    #[cfg(feature = "local")]
    let quote_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0],
    };

    #[cfg(feature = "testnet")]
    let quote_cell_type = ScriptLiteral {
        code_hash: [
            150, 36, 140, 222, 251, 9, 238, 217, 16, 1, 138, 132, 124, 251, 81, 173, 4, 76, 45,
            125, 182, 80, 17, 41, 49, 118, 14, 62, 243, 74, 126, 154,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0],
    };

    #[cfg(feature = "mainnet")]
    let quote_cell_type = ScriptLiteral {
        code_hash: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        hash_type: ScriptHashType::Type,
        args: vec![0],
    };

    util::script_literal_to_script(quote_cell_type)
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
129, 21, 231, 49, 194, 134, 216, 2, 221, 55, 127, 61, 130, 198, 227, 194, 21, 120, 185, 203, 22, 233, 16, 11, 78, 104, 64, 224, 240, 246, 149, 139
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
