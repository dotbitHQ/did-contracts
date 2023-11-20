use core::convert::TryFrom;

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
#[cfg(not(feature = "no_std"))]
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use super::schemas::packed::{Uint32, Uint32Reader};

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum SystemStatus {
    Off,
    On,
}

pub const ACCOUNT_ID_LENGTH: usize = 20;
pub const PRESERVED_ACCOUNT_CELL_COUNT: u8 = 20;

#[derive(Debug, PartialEq, Copy, Clone, TryFromPrimitive, EnumString, Display)]
#[cfg_attr(not(feature = "no_std"), derive(Eq, Hash))]
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

#[cfg(not(feature = "no_std"))]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u64)]
pub enum Source {
    Input = 1,
    Output = 2,
    CellDep = 3,
    HeaderDep = 4,
    GroupInput = 0x0100000000000001,
    GroupOutput = 0x0100000000000002,
}
