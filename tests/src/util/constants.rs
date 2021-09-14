use ckb_tool::ckb_types::{h256, H256};

// ⚠️ The maximum cycles on-chain is 70_000_000.
pub const MAX_CYCLES: u64 = u64::MAX;

pub const ACCOUNT_ID_LENGTH: usize = 20;
pub const ACCOUNT_BASIC_CAPACITY: u64 = 20_600_000_000;
pub const ACCOUNT_PREPARED_FEE_CAPACITY: u64 = 100_000_000;
pub const ACCOUNT_OPERATE_FEE: u64 = 10_000;
pub const ACCOUNT_RELEASED_LENGTH: usize = 5;

pub const ACCOUNT_PRICE_1_CHAR: u64 = 2000_000_000;
pub const ACCOUNT_PRICE_2_CHAR: u64 = 1000_000_000;
pub const ACCOUNT_PRICE_3_CHAR: u64 = 700_000_000;
pub const ACCOUNT_PRICE_4_CHAR: u64 = 170_000_000;
pub const ACCOUNT_PRICE_5_CHAR: u64 = 5_000_000;
pub const INVITED_DISCOUNT: u64 = 500;
pub const CONSOLIDATING_FEE: u64 = 100;
pub const CKB_QUOTE: u64 = 1000;

pub const RATE_BASE: u64 = 10_000;

// error numbers
pub const ERROR_EMPTY_ARGS: i8 = 5;

pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const SIGHASH_TYPE_HASH: H256 = h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
pub const MULTISIG_TYPE_HASH: H256 = h256!("0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8");
pub const DAO_TYPE_HASH: H256 = h256!("0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e");

// TODO Use another code hash instead of real type ID of secp256k1-blake160-signhash-all.
pub const ALWAYS_SUCCESS_TYPE_HASH: &str = "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";
pub const FAKE_DAS_LOCK_TYPE_HASH: &str = "0xcd289a6d68ca96b6b8df89e721aeb09350db5769a5e46908dfc797dbbf2a835f";
pub const FAKE_SIGNHASH_ALL_LOCK_TYPE_HASH: &str = "0xdc34ec56c0d6ec64c8f66f14dd53f1bcea08d54ed4e944606816b4ee95be9646";

pub const CONFIG_LOCK_ARGS: &str = "0x0000000000000000000000000000000000000000";
pub const QUOTE_LOCK_ARGS: &str = "0x0100000000000000000000000000000000000000";

// The type IDs here are testing only.
// fake-das-lock: 0xcd289a6d68ca96b6b8df89e721aeb09350db5769a5e46908dfc797dbbf2a835f
// fake-secp256k1-blake160-signhash-all: 0xdc34ec56c0d6ec64c8f66f14dd53f1bcea08d54ed4e944606816b4ee95be9646
pub const TYPE_ID_TABLE: [(&str, &str); 8] = [
    (
        "account-cell-type",
        "0x3d216e5bfb54b9e2ec0f0fbb1cdf23703f550a7ec7c35264742fce69308482e1",
    ),
    (
        "always-success",
        "0x3f67f5b5761db78ce746f0b140e0e63783fa84598e7e19a02ae8d417c0dfb882",
    ),
    (
        "apply-register-cell-type",
        "0xcac501b0a5826bffa485ccac13c2195fcdf3aa86b113203f620ddd34d3decd70",
    ),
    (
        "balance-cell-type",
        "0x3a36a0e90097a7d353bdb27f446b6b68759cfbc8282088d8d59926f271b324af",
    ),
    (
        "config-cell-type",
        "0x086BDCBEF0AB628D31AED1E7BAA26416D3BDE1E242A5A47DDDAEC06E87E595D0",
    ),
    (
        "income-cell-type",
        "0x3ff05cd948339d6b841487a288fbfa137e0f66c9eda15b62e71f3d3676d6395e",
    ),
    (
        "pre-account-cell-type",
        "0x431a3af2d4bbcd69ab732d37be794ac0ab172c151545dfdbae1f578a7083bc84",
    ),
    (
        "proposal-cell-type",
        "0x071ee1a005b5bc1a619aed290c39bbb613ac93991eabab8418d6b0a9bdd220eb",
    ),
];

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum Source {
    Input = 1,
    Output,
    CellDep,
    HeaderDep,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OracleCellType {
    Quote = 0,
    Time = 1,
    Height = 2,
}
