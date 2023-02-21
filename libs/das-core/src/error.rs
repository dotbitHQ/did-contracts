use alloc::boxed::Box;
use alloc::string::String;
use core::convert::Into;
use core::fmt;

use ckb_std::error::SysError;

/// Error
///
/// Error code range rules:
/// 1 ~ 50: reserved for common error
/// 50 ~ 126: shared by all type script
/// the rest: temporarily reserved
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum ErrorCode {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Customized errors:
    HardCodedError, // 5
    InvalidTransactionStructure,
    InvalidCellData,
    InitDayHasPassed,
    OracleCellIsRequired = 10,
    OracleCellDataDecodingError,
    ConfigTypeIsUndefined,
    ConfigIsPartialMissing,
    ConfigCellIsRequired,
    ConfigCellWitnessIsCorrupted,
    ConfigCellWitnessDecodingError,
    TxFeeSpentError,
    DasLockArgsInvalid,
    CellLockCanNotBeModified = 20,
    CellTypeCanNotBeModified,
    CellDataCanNotBeModified,
    CellCapacityMustReduced,
    CellCapacityMustIncreased,
    CellCapacityMustBeConsistent, // 25
    CellsMustHaveSameOrderAndNumber,
    ActionNotSupported,
    ParamsDecodingError,
    SuperLockIsRequired,
    AlwaysSuccessLockIsRequired, // 30
    SignallLockIsRequired,
    DataTypeUpgradeRequired,
    NarrowMixerTypeFailed,
    ChangeError,
    AccountStillCanNotBeRegister = 35, // ⚠️ DO NOT CHANGE
    AccountIsPreserved,
    AccountIsUnAvailable,
    AccountIdIsInvalid,
    WitnessStructureError = 40,
    WitnessDataTypeDecodingError,
    WitnessReadingError,
    WitnessActionDecodingError,
    WitnessDataParseLengthHeaderFailed,
    WitnessDataReadDataBodyFailed, // 45
    WitnessDataDecodingError,
    WitnessDataHashOrTypeMissMatch,
    WitnessDataIndexMissMatch,
    WitnessEntityDecodingError,
    WitnessEmpty, // 50
    WitnessArgsInvalid,
    WitnessArgsDecodingError,
    WitnessVersionOrTypeInvalid,
    SMTWhiteListTheLockIsNotFound,
    ApplyRegisterNeedWaitLonger = 60,
    ApplyRegisterHasTimeout,
    ApplyLockMustBeUnique,
    ApplyRegisterSinceMismatch,
    ApplyRegisterRefundCapacityError,
    CharSetIsConflict,
    CharSetIsUndefined,
    AccountCharIsInvalid,
    AccountIsTooLong,
    ProposalSliceIsNotSorted = 90,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalSliceRelatedCellMissing,
    ProposalCellTypeError, // 95
    ProposalCellAccountIdError,
    ProposalCellNextError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmNewAccountCellDataError = 100,
    ProposalConfirmNewAccountCellCapacityError,
    ProposalConfirmNewAccountWitnessError,
    ProposalConfirmPreAccountCellExpired,
    ProposalConfirmNeedWaitLonger,
    ProposalConfirmInitialRecordsMismatch,
    ProposalConfirmAccountLockArgsIsInvalid = 110,
    ProposalConfirmRefundError,
    ProposalSlicesCanNotBeEmpty,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell,
    ProposalSliceMustContainMoreThanOneElement, // 115
    ProposalSliceItemMustBeUniqueAccount,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleRefundAmountError,
    // 120
    PrevProposalItemNotFound,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    IncomeCellConsolidateError,
    IncomeCellConsolidateWaste,
    IncomeCellTransferError,
    IncomeCellCapacityError,
    IncomeCellProfitMismatch,
    EIP712SerializationError = -90,
    EIP712SematicError,
    EIP712DecodingWitnessArgsError,
    EIP712SignatureError,
    BalanceCellFoundSomeOutputsLackOfType = -80,
    BalanceCellCanNotBeSpent,
    AccountSaleCellCapacityError,
    AccountSaleCellRefundError,
    AccountSaleCellAccountIdInvalid,
    AccountSaleCellStartedAtInvalid,
    AccountSaleCellPriceTooSmall,
    AccountSaleCellDescriptionTooLarge,
    AccountSaleCellNewOwnerError,
    AccountSaleCellNotPayEnough,
    AccountSaleCellProfitError,
    AccountSaleCellProfitRateError,
    OfferCellCapacityError,
    OfferCellLockError,
    OfferCellMessageTooLong,
    OfferCellNewOwnerError,
    OfferCellFieldCanNotModified,
    OfferCellAccountMismatch,
    SubAccountFeatureNotEnabled = -50,
    SubAccountCellSMTRootError,
    SubAccountWitnessSMTRootError,
    SubAccountWitnessMismatched,
    SubAccountSignMintExpiredAtTooLarge,
    SubAccountSignMintExpiredAtReached,
    SubAccountSignMintSignatureRequired,
    SubAccountCellCapacityError,
    SubAccountCellAccountIdError,
    SubAccountCellConsistencyError,
    SubAccountInitialValueError,
    SubAccountSigVerifyError,
    SubAccountFieldNotEditable,
    SubAccountNormalCellLockLimit = -37,
    SubAccountEditLockError,
    SubAccountJoinBetaError,
    SubAccountProfitError,
    SubAccountCustomScriptError,
    SubAccountCollectProfitError,
    SubAccountBalanceManagerError,
    // -40
    UpgradeForWitnessIsRequired,
    UpgradeDefaultValueOfNewFieldIsError,
    CrossChainLockError,
    CrossChainUnlockError,
    UnittestError = -2,
    SystemOff = -1,
}

impl From<SysError> for ErrorCode {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<ErrorCode> for Box<dyn ScriptError> {
    fn from(err: ErrorCode) -> Box<dyn ScriptError> {
        Box::new(Error::new(err, String::new()))
    }
}

impl Into<i8> for ErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum AccountCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    AccountCellNextUpdateError = 50,
    AccountCellIdNotMatch,
    AccountCellPermissionDenied,
    AccountCellOwnerLockShouldNotBeModified,
    AccountCellOwnerLockShouldBeModified,
    AccountCellManagerLockShouldBeModified,
    AccountCellDataNotConsistent,
    AccountCellProtectFieldIsModified,
    AccountCellNoMoreFee,
    AccountCellRenewDurationMustLongerThanYear,
    // 60
    AccountCellRenewDurationBiggerThanPayed,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError,
    AccountCellRecordKeyInvalid,
    AccountCellRecordSizeTooLarge,
    AccountCellRecordNotEmpty,
    AccountCellStatusLocked,
    AccountCellIsNotExpired,
    AccountCellInExpirationAuctionConfirmationPeriod,
    AccountCellInExpirationAuctionPeriod,
    // 70
    AccountCellHasExpired,
    AccountCellStillCanNotRecycle,
}

impl From<SysError> for AccountCellErrorCode {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl Into<i8> for AccountCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum PreAccountCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    ApplyHashMismatch = 50,
    ApplySinceMismatch,
    AccountIdIsInvalid,
    AccountAlreadyExistOrProofInvalid,
    CreateAtIsInvalid,
    PriceIsInvalid,
    CharSetIsUndefined,
    CKBIsInsufficient,
    QuoteIsInvalid,
    OwnerLockArgsIsInvalid,
    RefundLockMustBeUnique,
    RefundCapacityError,
    SinceMismatch,
    InviterIdShouldBeEmpty,
    InviterIdIsInvalid,
    InviteeDiscountShouldBeEmpty,
    InviteeDiscountIsInvalid,
}

impl From<SysError> for PreAccountCellErrorCode {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl Into<i8> for PreAccountCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum ReverseRecordRootCellErrorCode {
    // WARNING Reserved errors:
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    AccountCellMissingPrevAccount = -114,
    AccountCellThrottle = -102,
    AccountCellInExpirationGracePeriod = -99,
    SubAccountNormalCellLockLimit = -37,
    SystemOff = -1,
    // Customized errors:
    InitialCapacityError = 50,
    InitialOutputsDataError,
    SMTNewRootMismatch,
}

impl From<SysError> for ReverseRecordRootCellErrorCode {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl Into<i8> for ReverseRecordRootCellErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}

pub trait ScriptError {
    fn as_i8(&self) -> i8;
}

#[derive(Debug, Clone)]
pub struct Error<T: Into<i8> + From<SysError> + Copy> {
    pub code: T,
    pub message: String,
}

impl<T: Into<i8> + From<SysError> + Copy> Error<T> {
    pub fn new(code: T, message: String) -> Self {
        Self { code, message }
    }

    pub fn boxed(code: T, message: String) -> Box<Self> {
        Box::new(Self { code, message })
    }
}

impl<T: Into<i8> + From<SysError> + Copy> ScriptError for Error<T> {
    fn as_i8(&self) -> i8 {
        self.code.into()
    }
}

impl<T: Into<i8> + From<SysError> + Copy> PartialEq for Error<T> {
    fn eq(&self, other: &Self) -> bool {
        self.code.into() == other.code.into()
    }
}

impl<'a, T: Into<i8> + From<SysError> + Copy + 'a> From<Error<T>> for Box<dyn ScriptError + 'a> {
    fn from(err: Error<T>) -> Box<dyn ScriptError + 'a> {
        Box::new(err)
    }
}

impl<'a, T: Into<i8> + From<SysError> + Copy + 'a> From<Box<Error<T>>> for Box<dyn ScriptError + 'a> {
    fn from(err: Box<Error<T>>) -> Box<dyn ScriptError + 'a> {
        err
    }
}

impl<T: Into<i8> + From<SysError> + Copy> From<SysError> for Error<T> {
    fn from(err: SysError) -> Self {
        Self::new(T::from(err), String::new())
    }
}

impl From<SysError> for Box<dyn ScriptError> {
    fn from(err: SysError) -> Box<dyn ScriptError> {
        Box::new(Error::<ErrorCode>::from(err))
    }
}

impl fmt::Debug for Box<dyn ScriptError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Box<dyn ScriptError>")
            .field("code", &self.as_i8())
            .finish()
    }
}
