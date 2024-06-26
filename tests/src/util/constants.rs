use std::collections::HashMap;

use ckb_types::{h256, H256};
use lazy_static::lazy_static;
use regex::Regex;

// ⚠️ The maximum cycles on-chain is 70_000_000.
pub const MAX_CYCLES: u64 = u64::MAX;

pub const USD_1: u64 = 1_000_000;
pub const USD_5: u64 = 5 * USD_1;
pub const USD_10: u64 = 10 * USD_1;
pub const USD_20: u64 = 20 * USD_1;

pub const ONE_CKB: u64 = 100_000_000;
pub const CKB_QUOTE: u64 = 1000;

pub const APPLY_MIN_WAITING_BLOCK: u64 = 1;
pub const APPLY_MAX_WAITING_BLOCK: u64 = 5760;
pub const APPLY_REFUND_REWARD: u64 = 100_000_000;

pub const ACCOUNT_BASIC_CAPACITY: u64 = 20_600_000_000;
pub const ACCOUNT_PREPARED_FEE_CAPACITY: u64 = 100_000_000;
pub const ACCOUNT_OPERATE_FEE: u64 = 10_000;
pub const ACCOUNT_EXPIRATION_GRACE_PERIOD: u64 = 7_776_000;
pub const ACCOUNT_EXPIRATION_AUCTION_PERIOD: u64 = 2_588_400;
pub const ACCOUNT_EXPIRATION_DELIVER_PERIOD: u64 = 3600;
pub const ACCOUNT_EXPIRATION_AUCTION_START_PREMIUMS: u64 = 100_000_000;

pub const ACCOUNT_PRICE_1_CHAR: u64 = 0;
pub const ACCOUNT_PRICE_2_CHAR: u64 = 1000_000_000;
pub const ACCOUNT_PRICE_3_CHAR: u64 = 700_000_000;
pub const ACCOUNT_PRICE_4_CHAR: u64 = 160_000_000;
pub const ACCOUNT_PRICE_5_CHAR: u64 = 5_000_000;
pub const INVITED_DISCOUNT: u64 = 500;
pub const CONSOLIDATING_FEE: u64 = 100;
pub const TIMESTAMP: u64 = 1611200090u64;
pub const TIMESTAMP_20221018: u64 = 1666094400u64;
pub const HEIGHT: u64 = 1000000u64;

pub const PRE_ACCOUNT_TIMEOUT_LIMIT: u64 = 1 * DAY_SEC;
pub const PRE_ACCOUNT_SHORT_TIMEOUT_LIMIT: u64 = HOUR_SEC;
pub const PRE_ACCOUNT_REFUND_AVAILABLE_FEE: u64 = DAY_SEC;

pub const INCOME_BASIC_CAPACITY: u64 = 20_000_000_000;

pub const SALE_BUYER_INVITER_PROFIT_RATE: u64 = 100;
pub const SALE_BUYER_CHANNEL_PROFIT_RATE: u64 = 100;
pub const SALE_DAS_PROFIT_RATE: u64 = 100;

pub const ACCOUNT_SALE_MIN_PRICE: u64 = 20_000_000_000;
pub const ACCOUNT_SALE_BASIC_CAPACITY: u64 = 20_000_000_000;
pub const ACCOUNT_SALE_PREPARED_FEE_CAPACITY: u64 = 100_000_000;
pub const OFFER_BASIC_CAPACITY: u64 = 20_000_000_000;
pub const OFFER_PREPARED_FEE_CAPACITY: u64 = 100_000_000;
pub const OFFER_PREPARED_MESSAGE_BYTES_LIMIT: u64 = 5000;
pub const SECONDARY_MARKET_COMMON_FEE: u64 = 10_000;

pub const REVERSE_RECORD_BASIC_CAPACITY: u64 = 20_000_000_000;
pub const REVERSE_RECORD_PREPARED_FEE_CAPACITY: u64 = 100_000_000;
pub const REVERSE_RECORD_COMMON_FEE: u64 = 10_000;

pub const SUB_ACCOUNT_BASIC_CAPACITY: u64 = 20_000_000_000;
pub const SUB_ACCOUNT_PREPARED_FEE_CAPACITY: u64 = 1_000_000_000;
pub const SUB_ACCOUNT_NEW_PRICE: u64 = 990_000;
pub const SUB_ACCOUNT_NEW_CUSTOM_PRICE: u64 = 5_000_000;
pub const SUB_ACCOUNT_NEW_CUSTOM_PRICE_DAS_PROFIT_RATE: u64 = 2_000;
pub const SUB_ACCOUNT_RENEW_PRICE: u64 = 990_000;
pub const SUB_ACCOUNT_RENEW_CUSTOM_PRICE_DAS_PROFIT_RATE: u64 = 2_000;
pub const SUB_ACCOUNT_COMMON_FEE: u64 = 30_000;
pub const SUB_ACCOUNT_CREATE_FEE: u64 = 30_000;
pub const SUB_ACCOUNT_EDIT_FEE: u64 = 30_000;
pub const SUB_ACCOUNT_RENEW_FEE: u64 = 30_000;
pub const SUB_ACCOUNT_RECYCLE_FEE: u64 = 30_000;

pub const DPOINT_BASIC_CAPACITY: u64 = 20_000_000_000;
pub const DPOINT_PREPARED_FEE_CAPACITY: u64 = 0;

pub const HOUR_SEC: u64 = 3600;
pub const DAY_SEC: u64 = 86400;
pub const MONTH_SEC: u64 = DAY_SEC * 30;
pub const YEAR_SEC: u64 = DAY_SEC * 365;

pub const RATE_BASE: u64 = 10_000;

// error numbers
pub const ERROR_EMPTY_ARGS: i8 = 5;

pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const SIGHASH_TYPE_HASH: H256 = h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
pub const MULTISIG_TYPE_HASH: H256 = h256!("0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8");
pub const DAO_TYPE_HASH: H256 = h256!("0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e");
pub const BLACK_HOLE_HASH: H256 = h256!("0x0000000000000000000000000000000000000000000000000000000000000000");

pub const CONFIG_LOCK_ARGS: &str = "0x0000000000000000000000000000000000000000";
pub const DAS_WALLET_LOCK_ARGS: &str = "0x0300000000000000000000000000000000000000";
pub const QUOTE_LOCK_ARGS: &str = "0x0100000000000000000000000000000000000000";
pub const PROFIT_LOCK_ARGS: &str = "0x0400000000000000000000000000000000000000";
pub const DUMMY_LOCK_ARGS: &str = "0xff00000000000000000000000000000000000000";

#[derive(Debug)]
#[repr(u8)]
pub enum ScriptHashType {
    Data = 0,
    Type = 1,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OracleCellType {
    Quote = 0,
    Time = 1,
    Height = 2,
}

lazy_static! {
    pub static ref TYPE_ID_TABLE: HashMap<&'static str, &'static str> = {
        // For calculation of these type ID, you need uncomment a line of debug code in the funtion **mock_contract** in src/util/template_parser .
        //
        // CAREFUL! There may be some error in the map, but the contracts will still work. It is because when parsing scripts in cell_deps, their type
        // ID will be calculated dynamically and insert into the map.
        let mut map = HashMap::new();
        // fake locks
        map.insert(
            "fake-das-lock",
            "0xebd2ca43797df1eae21f5a0d20a09a3851beab063ca06d7b86a1e1e8ef9c7698",
        );
        map.insert(
            "fake-secp256k1-blake160-signhash-all",
            "0x8f2d7cb06512f2777207461d100b0562b0213232a1bd70261e57f37fdc61483d",
        );
        map.insert(
            "always_success",
            "0x34f052fc455fce7c71f4905f223653a5fbe64261c6b2537124de00f1d52820e9",
        );

        map.insert(
            "always-success",
            "0x610b14e8060fca49a46606bf2eaaa01f77a77daf27c22a3bec3cd13c6ceb1a60",
        );
        // types
        map.insert(
            "account-cell-type",
            "0x8974b2101b074d7cd80ffb780c21758883fcc007fe7c39cf6556e09d2bdfd3ef",
        );
        map.insert(
            "account-sale-cell-type",
            "0x1d39d4f27b890bd91d7ae7d040ae33c81f391705f5742979ce9aaf10a242f473",
        );
        map.insert(
            "account-auction-cell-type",
            "0x3acbbdc4c0f0dc7433f5aac30b079a3fd3bfaaf3aeeea904af830dad99da1e49",
        );
        map.insert(
            "apply-register-cell-type",
            "0x4712c67d41ca8071394c92cc7a022b853ffb877692ca5f473535d169555b2c46",
        );
        map.insert(
            "balance-cell-type",
            "0xfab0668d7e96e5ea3e52f9286e854c557aa10c4a33fd1f91be8a47ed94ea9e75",
        );
        map.insert(
            "config-cell-type",
            "0x7b8cd34cd5e3374aa9dfac108cf12336e931933e892f54471e469fc1b31a3cca",
        );
        map.insert(
            "income-cell-type",
            "0xdf11093f25adecd27f02170ce8c5fd15cd88094416a9731d0ec20cd4729f4cdc",
        );
        map.insert(
            "offer-cell-type",
            "0x78fc7cd320243aede8bfb4eb70ad53804e6d36c24cad3a4b728439192e5425cb",
        );
        map.insert(
            "pre-account-cell-type",
            "0xf85ef3af97458169e2cca2a3faf296ac49fd3d2dea90fd35d3f9df09ab0375ea",
        );
        map.insert(
            "proposal-cell-type",
            "0xebe59b46a2e053394ad18c97109783126fca6754e1d9b8d4313155da8a148e21",
        );
        map.insert(
            "reverse-record-cell-type",
            "0x573b1d865799c4bcb98ebd8b75bd87ed6a6c2449c99edf0f17142f527118201e",
        );
        map.insert(
            "reverse-record-root-cell-type",
            "0x49dc1d7f8c02869636f60b7eaea825cdc0d8daf68a3d902532fd8f24ecdd628a",
        );
        map.insert(
            "sub-account-cell-type",
            "0xf70fb11157496e73f30fc5e781d52725a74c9fba1e7a52115d75320d171759ec",
        );
        // libs
        map.insert(
            "eip712-lib",
            "0x72ed0770c719091b424b7072fe69dc362fb8867e11df39ac6801557d8e559fcd",
        );
        // others
        map.insert(
            "test-env",
            "0x444c2ed8b24700700fcac7cdc0989e1db41380b9c79cc1cf30159e5336ba7d4a",
        );
        map.insert(
            "playground",
            "0xca4d966895b1467702bad4038396b037d8c8f045cae9cf5a7db4eadefa347887",
        );
        // das-lock
        map.insert(
            "ckb_sign.so",
            "0x84797b60caf864addda007c6e16f570a8709d6fa27b32f4a093e82a6b7565817",
        );
        map.insert(
            "ckb_multi_sign.so",
            "0x0edf7093a5202d33ae1083947823f706a68e7af4b7641b5d88bbf3fd6af15153",
        );
        map.insert(
            "ed25519_sign.so",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        );
        map.insert(
            "eth_sign.so",
            "0xb16a069e007b20528efeb0c0c7742bfee94d6e0e15eb07b5f7e2343dcecce768",
        );
        map.insert(
            "tron_sign.so",
            "0xed45ad69b0ee84b7b8f0181f75eda16948682438a40a0f7597667bfab5fd89e5",
        );
        map.insert(
            "doge_sign.so",
            "0x0480767c09c1c26abe4e180d08238ed551f11dc293ab1286d8a7b7cac95e55a7",
        );
        map.insert(
            "webauthn_sign.so",
            "0xa22105572bf4c7c1c3bcfb97bcc1d3bba060f32c50df5364ee1cb2b09228d7aa"
        );
        map.insert(
            "device-key-list-cell-type",
            "0x982f5e9cc7503f4cd0d0e2ed7115b70f9007f6adcdbdefb5465228d39c94f12a"
        );
        map.insert(
            "dpoint-cell-type",
            "0x2de294cbd18800cb5cee3bb264e85d376922938c8726e40d89c5ee59d3695afd"
        );
        map
    };
    pub static ref RE_VARIABLE: Regex = Regex::new(r"\{\{([\w\-\.]+)\}\}").unwrap();
    pub static ref RE_ZH_CHAR: Regex = Regex::new(r"^[\u4E00-\u9FA5]+$").unwrap();
}
