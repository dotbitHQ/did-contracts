// Suppress warning here is because it is mistakenly treat the code as dead code when running unit tests.
#![allow(dead_code)]

use ckb_tool::ckb_types::{h256, H256};

pub const MAX_CYCLES: u64 = 10_000_000;

// error numbers
pub const ERROR_EMPTY_ARGS: i8 = 5;

pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const SIGHASH_TYPE_HASH: H256 =
    h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
pub const MULTISIG_TYPE_HASH: H256 =
    h256!("0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8");
pub const DAO_TYPE_HASH: H256 =
    h256!("0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e");
