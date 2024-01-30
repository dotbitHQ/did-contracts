pub use ckb_std::ckb_types::core::ScriptHashType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Debug)]
pub enum LockScript {
    AlwaysSuccessLock,
    DasLock,
    Secp256k1Blake160SignhashLock,
    Secp256k1Blake160MultisigLock,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OracleCellType {
    Quote = 0,
    Time = 1,
    Height = 2,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum CellField {
    Capacity,
    Lock,
    Type,
    Data,
}

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const ONE_CKB: u64 = 100_000_000;
pub const CELL_BASIC_CAPACITY: u64 = 6_1 * ONE_CKB;
pub const ONE_USD: u64 = 1_000_000;
pub const DPOINT_MAX_LIMIT: u64 = 10_000_000 * ONE_USD;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_SUFFIX: &str = ".bit";
pub const ACCOUNT_MAX_PRICED_LENGTH: u8 = 8;

pub const CUSTOM_KEYS_NAMESPACE: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz_";
pub const COIN_TYPE_DIGITS: &[u8] = b"0123456789";

pub const SECP_SIGNATURE_SIZE: usize = 65;
// This is smaller than the real data type in solidity, but it is enough for now.
pub const EIP712_CHAINID_SIZE: usize = 8;

pub const DAY_SEC: u64 = 86400;
pub const DAYS_OF_YEAR: u64 = 365;
pub const YEAR_SEC: u64 = DAY_SEC * DAYS_OF_YEAR;

pub const PRE_ACCOUNT_CELL_TIMEOUT: u64 = DAY_SEC;
pub const PRE_ACCOUNT_CELL_SHORT_TIMEOUT: u64 = 3600;

pub const CROSS_CHAIN_BLACK_ARGS: [u8; 20] = [0; 20];

pub const TYPE_ID_CODE_HASH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 89, 80, 69, 95, 73, 68,
];
