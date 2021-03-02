use ckb_tool::ckb_types::{h256, H256};

pub const MAX_CYCLES: u64 = 100_000_000; // up to 70_000_000

// error numbers
pub const ERROR_EMPTY_ARGS: i8 = 5;

pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const SIGHASH_TYPE_HASH: H256 =
    h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
pub const MULTISIG_TYPE_HASH: H256 =
    h256!("0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8");
pub const DAO_TYPE_HASH: H256 =
    h256!("0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e");

pub const ALWAYS_SUCCESS_CODE_HASH: [u8; 32] = [
    157, 111, 41, 25, 227, 40, 243, 33, 125, 125, 211, 218, 181, 247, 206, 233, 216, 224, 98, 190,
    230, 168, 13, 93, 5, 205, 73, 92, 163, 65, 99, 120,
];

pub const CONFIG_LOCK_ARGS: &str = "0x0000000000000000000000000000000000000000";
pub const QUOTE_LOCK_ARGS: &str = "0x0100000000000000000000000000000000000000";

// The type IDs here are testing only.
pub const TYPE_ID_TABLE: [(&str, &str); 7] = [
    (
        "apply-register-cell-type",
        "0xcac501b0a5826bffa485ccac13c2195fcdf3aa86b113203f620ddd34d3decd70",
    ),
    (
        "config-cell-type",
        "0x086BDCBEF0AB628D31AED1E7BAA26416D3BDE1E242A5A47DDDAEC06E87E595D0",
    ),
    (
        "pre-account-cell-type",
        "0x431a3af2d4bbcd69ab732d37be794ac0ab172c151545dfdbae1f578a7083bc84",
    ),
    (
        "account-cell-type",
        "0x3d216e5bfb54b9e2ec0f0fbb1cdf23703f550a7ec7c35264742fce69308482e1",
    ),
    (
        "proposal-cell-type",
        "0x071ee1a005b5bc1a619aed290c39bbb613ac93991eabab8418d6b0a9bdd220eb",
    ),
    (
        "ref-cell-type",
        "0x15f69a14cfafac4e21516e7076e135492c4b20fe4fb5af9e1942577a46985a13",
    ),
    (
        "wallet-cell-type",
        "0xcf2f19e19c13d4ccfeae96634f6be6cdb2e4cd68f810ce3b865ee34030374524",
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
