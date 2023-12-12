#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
use core::cell::OnceCell;
use core::convert::TryFrom;
use core::env;

#[cfg(feature = "no_std")]
use ckb_std::ckb_constants::Source as CkbSource;
#[cfg(feature = "no_std")]
use ckb_std::ckb_types::core::ScriptHashType;
#[cfg(feature = "no_std")]
use ckb_std::ckb_types::packed::Byte;
#[cfg(not(feature = "no_std"))]
use ckb_types::core::ScriptHashType;
#[cfg(not(feature = "no_std"))]
use ckb_types::packed::Byte;
use molecule::prelude::{Builder, Entity};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
#[cfg(not(feature = "no_std"))]
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use super::schemas::packed::{self, Hash, Script, Uint32, Uint32Reader};

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SystemStatus {
    Off,
    On,
}

pub const ACCOUNT_ID_LENGTH: usize = 20;
pub const PRESERVED_ACCOUNT_CELL_COUNT: u8 = 20;

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

#[derive(Debug, Copy, Clone, TryFromPrimitive, EnumString, Display, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(feature = "no_std"), derive(Hash))]
#[repr(u32)]
pub enum DataType {
    ActionData = 0,
    AccountCellData,
    AccountSaleCellData,
    AccountAuctionCellData,
    ProposalCellData,
    PreAccountCellData,
    IncomeCellData,
    OfferCellData,
    SubAccount,
    SubAccountMintSign,
    ReverseRecord,
    SubAccountPriceRule,
    SubAccountPreservedRule,
    DeviceKeyListEntityData,
    SubAccountRenewSign,
    DeviceKeyListCellData,
    ConfigCellAccount = 100,              // args: 0x64000000
    ConfigCellApply = 101,                // args: 0x65000000
    ConfigCellIncome = 103,               // args: 0x67000000
    ConfigCellMain,                       // args: 0x68000000
    ConfigCellPrice,                      // args: 0x69000000
    ConfigCellProposal,                   // args: 0x6a000000
    ConfigCellProfitRate,                 // args: 0x6b000000
    ConfigCellRecordKeyNamespace,         // args: 0x6c000000
    ConfigCellRelease,                    // args: 0x6d000000
    ConfigCellUnAvailableAccount,         // args: 0x6e000000
    ConfigCellSecondaryMarket,            // args: 0x6f000000
    ConfigCellReverseResolution,          // args: 0x70000000
    ConfigCellSubAccount,                 // args: 0x71000000
    ConfigCellSubAccountBetaList,         // args: 0x72000000
    ConfigCellSystemStatus,               // args: 0x73000000
    ConfigCellSMTNodeWhitelist,           // args: 0x74000000
    ConfigCellDPoint,                     // args: 0x75000000
    ConfigCellPreservedAccount00 = 10000, // args: 0x10270000
    ConfigCellPreservedAccount01,
    ConfigCellPreservedAccount02,
    ConfigCellPreservedAccount03,
    ConfigCellPreservedAccount04,
    ConfigCellPreservedAccount05,
    ConfigCellPreservedAccount06,
    ConfigCellPreservedAccount07,
    ConfigCellPreservedAccount08,
    ConfigCellPreservedAccount09,
    ConfigCellPreservedAccount10,
    ConfigCellPreservedAccount11,
    ConfigCellPreservedAccount12,
    ConfigCellPreservedAccount13,
    ConfigCellPreservedAccount14,
    ConfigCellPreservedAccount15,
    ConfigCellPreservedAccount16,
    ConfigCellPreservedAccount17,
    ConfigCellPreservedAccount18,
    ConfigCellPreservedAccount19,     // args: 0x23270000
    ConfigCellCharSetEmoji = 100000,  // args: 0xa0860100
    ConfigCellCharSetDigit = 100001,  // args: 0xa1860100
    ConfigCellCharSetEn = 100002,     // args: 0xa2860100
    ConfigCellCharSetZhHans = 100003, // args: 0xa3860100, not available yet
    ConfigCellCharSetZhHant = 100004, // args: 0xa4860100, not available yet
    ConfigCellCharSetJa,
    ConfigCellCharSetKo,
    ConfigCellCharSetRu,
    ConfigCellCharSetTr,
    ConfigCellCharSetTh,
    ConfigCellCharSetVi,
    OrderInfo = 199999,
}

impl TryFrom<Uint32> for DataType {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(v: Uint32) -> Result<Self, Self::Error> {
        Self::try_from(u32::from(v))
    }
}

impl<'r> TryFrom<Uint32Reader<'r>> for DataType {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(v: Uint32Reader) -> Result<Self, Self::Error> {
        Self::try_from(u32::from(v))
    }
}

// The length of CharSetType
pub const CHAR_SET_LENGTH: usize = 11;

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive, EnumString, Display)]
#[cfg_attr(not(feature = "no_std"), derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum CharSetType {
    Emoji,
    Digit,
    En,
    ZhHans,
    ZhHant,
    Ja,
    Ko,
    Ru,
    Tr,
    Th,
    Vi, // ⚠️ DO NOT Forget to update CHAR_SET_LENGTH at the same time.
}

impl Default for CharSetType {
    fn default() -> Self {
        CharSetType::En
    }
}

impl TryFrom<Uint32> for CharSetType {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(v: Uint32) -> Result<Self, Self::Error> {
        Self::try_from(u32::from(v))
    }
}

impl<'r> TryFrom<Uint32Reader<'r>> for CharSetType {
    type Error = TryFromPrimitiveError<Self>;

    fn try_from(v: Uint32Reader) -> Result<Self, Self::Error> {
        Self::try_from(u32::from(v))
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum ProposalSliceItemType {
    Exist,
    Proposed,
    New,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum AccountStatus {
    Normal,
    Selling,
    Auction,
    LockedForCrossChain,
    ApprovedTransfer,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SubAccountEnableStatus {
    Off,
    On,
}

#[derive(Debug, Clone)]
pub enum TypeScript {
    AccountCellType,
    AccountSaleCellType,
    AccountAuctionCellType,
    ApplyRegisterCellType,
    BalanceCellType,
    ConfigCellType,
    IncomeCellType,
    OfferCellType,
    PreAccountCellType,
    ProposalCellType,
    ReverseRecordCellType,
    SubAccountCellType,
    ReverseRecordRootCellType,
    DPointCellType,
    EIP712Lib,
}

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive, Display)]
#[repr(u8)]
pub enum DasLockType {
    XXX,
    CKBMulti,
    CKBSingle,
    ETH,
    TRON,
    ETHTypedData,
    MIXIN,
    Doge,
    WebAuthn,
}

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum LockRole {
    Owner,
    Manager,
}

#[derive(Debug, PartialEq, Copy, Clone, EnumString, Display)]
pub enum SubAccountAction {
    #[strum(serialize = "create")]
    Create,
    #[strum(serialize = "edit")]
    Edit,
    #[strum(serialize = "renew")]
    Renew,
    #[strum(serialize = "recycle")]
    Recycle,
    #[strum(serialize = "create_approval")]
    CreateApproval,
    #[strum(serialize = "delay_approval")]
    DelayApproval,
    #[strum(serialize = "revoke_approval")]
    RevokeApproval,
    #[strum(serialize = "fulfill_approval")]
    FulfillApproval,
}

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive, Display)]
#[repr(u8)]
pub enum SubAccountConfigFlag {
    Manual,
    CustomScript, // deprecated
    CustomRule = 255,
}

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive, Display)]
#[repr(u8)]
pub enum SubAccountCustomRuleFlag {
    Off,
    On,
}

#[derive(Debug, Clone, PartialEq, EnumString, Display)]
pub enum ReverseRecordAction {
    #[strum(serialize = "update")]
    Update,
    #[strum(serialize = "remove")]
    Remove,
}

#[derive(Debug, Clone, PartialEq, EnumString, Display)]
pub enum AccountApprovalAction {
    #[strum(serialize = "transfer")]
    Transfer,
}

// [100, 97, 115] equals b"das"
pub const WITNESS_HEADER: [u8; 3] = [100, 97, 115];
pub const WITNESS_HEADER_BYTES: usize = WITNESS_HEADER.len();
pub const WITNESS_TYPE_BYTES: usize = 4;
pub const WITNESS_LENGTH_BYTES: usize = 4;
pub const SUB_ACCOUNT_WITNESS_VERSION_BYTES: usize = 8;
// WARNING! This constant maybe need to be enlarger in the future.
pub const SUB_ACCOUNT_WITNESS_ACTION_BYTES: usize = 4 + 20;
pub const REVERSE_RECORD_WITNESS_VERSION_BYTES: usize = 8;
// WARNING! This constant maybe need to be enlarger in the future.
pub const REVERSE_RECORD_WITNESS_ACTION_BYTES: usize = 4 + 10;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Source {
    Input = 1,
    Output = 2,
    CellDep = 3,
}

#[cfg(feature = "no_std")]
impl From<CkbSource> for Source {
    fn from(source: CkbSource) -> Self {
        match source {
            CkbSource::Input => Source::Input,
            CkbSource::Output => Source::Output,
            CkbSource::CellDep => Source::CellDep,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "no_std")]
impl Into<CkbSource> for Source {
    fn into(self) -> CkbSource {
        match self {
            Source::Input => CkbSource::Input,
            Source::Output => CkbSource::Output,
            Source::CellDep => CkbSource::CellDep,
        }
    }
}

#[derive(Debug, Default, PartialEq, EnumString, Display)]
pub enum Action {
    #[strum(serialize = "apply_register")]
    ApplyRegister,
    #[strum(serialize = "refund_apply")]
    RefundApply,
    #[strum(serialize = "pre_register")]
    PreRegister,
    #[strum(serialize = "confirm_proposal")]
    ConfirmProposal,
    #[strum(serialize = "renew_account")]
    RenewAccount,
    #[strum(serialize = "accept_offer")]
    AcceptOffer,
    #[strum(serialize = "unlock_account_for_cross_chain")]
    UnlockAccountForCrossChain,
    #[strum(serialize = "force_recover_account_status")]
    ForceRecoverAccountStatus,
    #[strum(serialize = "recycle_expired_account")]
    RecycleExpiredAccount,
    #[strum(serialize = "edit_records")]
    EditRecords,
    #[strum(serialize = "create_sub_account")]
    CreateSubAccount,
    #[strum(serialize = "update_sub_account")]
    UpdateSubAccount,
    #[strum(serialize = "config_sub_account")]
    ConfigSubAccount,
    #[strum(serialize = "config_sub_account_custom_script")]
    ConfigSubAccountCustomScript,
    #[strum(serialize = "buy_account")]
    BuyAccount,
    #[strum(serialize = "enable_sub_account")]
    EnableSubAccount,
    #[strum(serialize = "revoke_approval")]
    RevokeApproval,
    #[strum(serialize = "fulfill_approval")]
    FulfillApproval,
    #[strum(serialize = "bid_expired_account_dutch_auction")]
    BidExpiredAccountDutchAuction,
    #[strum(serialize = "lock_account_for_cross_chain")]
    LockAccountForCrossChain,
    #[default]
    Others,
    // Unit test only,
    #[strum(serialize = "test_parse_witness_entity_config")]
    TestParseWitnessEntityConfig,
    #[strum(serialize = "test_parse_witness_raw_config")]
    TestParseWitnessRawConfig,
    #[strum(serialize = "test_parse_witness_cells")]
    TestParseWitnessCells,
    #[strum(serialize = "test_parse_sub_account_witness_empty")]
    TestParseSubAccountWitnessEmpty,
    #[strum(serialize = "test_parse_sub_account_witness_create_only")]
    TestParseSubAccountWitnessCreateOnly,
    #[strum(serialize = "test_parse_sub_account_witness_edit_only")]
    TestParseSubAccountWitnessEditOnly,
    #[strum(serialize = "test_parse_sub_account_witness_mixed")]
    TestParseSubAccountWitnessMixed,
    #[strum(serialize = "test_parser_sub_account_rules_witness_empty")]
    TestParserSubAccountRulesWitnessEmpty,
    #[strum(serialize = "test_parser_sub_account_rules_witness")]
    TestParserSubAccountRulesWitness,
    #[strum(serialize = "test_parse_reverse_record_witness_empty")]
    TestParseReverseRecordWitnessEmpty,
    #[strum(serialize = "test_parse_reverse_record_witness_update_only")]
    TestParseReverseRecordWitnessUpdateOnly,
    #[strum(serialize = "test_parse_reverse_record_witness_remove_only")]
    TestParseReverseRecordWitnessRemoveOnly,
    #[strum(serialize = "test_parse_reverse_record_witness_mixed")]
    TestParseReverseRecordWitnessMixed,
    #[strum(serialize = "test_dotenv_loaded_properly")]
    TestDotEnvLoadedProperly,
}

#[derive(Debug, Default, PartialEq)]
pub enum ActionParams {
    LockAccountForCrossChain {
        coin_type: u64,
        chain_id: u64,
        role: LockRole,
    },
    BuyAccount {
        inviter_lock_args: Vec<u8>,
        channel_lock_args: Vec<u8>,
        role: LockRole,
    },
    Role(LockRole),
    #[default]
    None,
}

pub fn super_lock() -> &'static Script {
    static mut SUPER_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("SUPER_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The SUPER_CODE_HASH should be a hex string.");

    let args = env!("SUPER_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The SUPER_ARGS should be a hex string.");

    unsafe {
        SUPER_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn wallet_lock() -> &'static Script {
    static mut WALLET_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("WALLET_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The WALLET_CODE_HASH should be a hex string.");

    let args = env!("WALLET_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The WALLET_ARGS should be a hex string.");

    unsafe {
        WALLET_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn cross_chain_lock() -> &'static Script {
    static mut CROSS_CHAIN_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("CROSS_CHAIN_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The CROSS_CHAIN_CODE_HASH should be a hex string.");

    let args = env!("CROSS_CHAIN_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The CROSS_CHAIN_ARGS should be a hex string.");

    unsafe {
        CROSS_CHAIN_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn profit_manager_lock() -> &'static Script {
    static mut PROFIT_MANAGER_LOCK: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("PROFIT_MANAGER_CODE_HASH").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The PROFIT_MANAGER_CODE_HASH should be a hex string.");

    let args = env!("PROFIT_MANAGER_ARGS").trim_start_matches("0x");
    let args = hex::decode(args).expect("The PROFIT_MANAGER_ARGS should be a hex string.");

    unsafe {
        PROFIT_MANAGER_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn das_lock() -> &'static Script {
    static mut DAS_LOCK: OnceCell<Script> = OnceCell::new();

    let type_id = env!("DAS_LOCK_TYPE_ID").trim_start_matches("0x");
    let type_id = hex::decode(type_id).expect("The DAS_LOCK_TYPE_ID should be a hex string.");

    unsafe {
        DAS_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}

pub fn always_success_lock() -> &'static Script {
    static mut ALWAYS_SUCCESS_LOCK: OnceCell<Script> = OnceCell::new();

    let type_id = env!("ALWAYS_SUCCESS_LOCK_TYPE_ID").trim_start_matches("0x");
    let type_id = hex::decode(type_id).expect("The ALWAYS_SUCCESS_LOCK_TYPE_ID should be a hex string.");

    unsafe {
        ALWAYS_SUCCESS_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}

pub fn signhash_lock() -> &'static Script {
    static mut SIGNHASH_LOCK: OnceCell<Script> = OnceCell::new();

    let type_id = env!("SIGNHASH_LOCK_TYPE_ID").trim_start_matches("0x");
    let type_id = hex::decode(type_id).expect("The SIGNHASH_LOCK_TYPE_ID should be a hex string.");

    unsafe {
        SIGNHASH_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}

pub fn multisign_lock() -> &'static Script {
    static mut MULTISIGN_LOCK: OnceCell<Script> = OnceCell::new();

    let type_id = env!("MULTISIG_LOCK_TYPE_ID").trim_start_matches("0x");
    let type_id = hex::decode(type_id).expect("The MULTISIG_LOCK_TYPE_ID should be a hex string.");

    unsafe {
        MULTISIGN_LOCK.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}

pub fn config_cell_type() -> &'static Script {
    static mut CONFIG_CELL_TYPE: OnceCell<Script> = OnceCell::new();

    let type_id = env!("CONFIG_CELL_TYPE_ID").trim_start_matches("0x");
    let type_id = hex::decode(type_id).expect("The CONFIG_CELL_TYPE_ID should be a hex string.");

    unsafe {
        CONFIG_CELL_TYPE.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(type_id).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .build();
            script
        })
    }
}

pub fn quote_cell_type() -> &'static Script {
    static mut QUOTE_CELL_TYPE: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("ORACLE_CELL_TYPE_ID").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The ORACLE_CELL_TYPE_ID should be a hex string.");

    let args = vec![0u8];

    unsafe {
        QUOTE_CELL_TYPE.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn time_cell_type() -> &'static Script {
    static mut TIME_CELL_TYPE: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("ORACLE_CELL_TYPE_ID").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The ORACLE_CELL_TYPE_ID should be a hex string.");

    let args = vec![1u8];

    unsafe {
        TIME_CELL_TYPE.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}

pub fn height_cell_type() -> &'static Script {
    static mut HEIGHT_CELL_TYPE: OnceCell<Script> = OnceCell::new();

    let code_hash = env!("ORACLE_CELL_TYPE_ID").trim_start_matches("0x");
    let code_hash = hex::decode(code_hash).expect("The ORACLE_CELL_TYPE_ID should be a hex string.");

    let args = vec![2u8];

    unsafe {
        HEIGHT_CELL_TYPE.get_or_init(|| {
            let script = Script::new_builder()
                .code_hash(Hash::try_from(code_hash).unwrap())
                .hash_type(Byte::new(ScriptHashType::Type.into()))
                .args(packed::Bytes::from(args))
                .build();
            script
        })
    }
}
