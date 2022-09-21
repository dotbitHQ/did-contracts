use alloc::{boxed::Box, string::String};
use ckb_std::error::SysError;
use core::convert::Into;
use core::fmt;

/// Error
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
    CellCapacityMustConsistent, // 25
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
    ApplyRegisterNeedWaitLonger = 60,
    ApplyRegisterHasTimeout,
    ApplyRegisterRefundNeedWaitLonger,
    ApplyRegisterRefundCapacityError,
    PreRegisterFoundInvalidTransaction = 70,
    PreRegisterAccountIdIsInvalid,
    PreRegisterApplyHashIsInvalid,
    PreRegisterCreateAtIsInvalid,
    PreRegisterPriceInvalid,
    CharSetIsUndefined, // 75
    PreRegisterCKBInsufficient,
    PreRegisterAccountIsTooLong,
    PreRegisterAccountCharSetConflict,
    PreRegisterAccountCharIsInvalid,
    PreRegisterQuoteIsInvalid, // 80
    PreRegisterDiscountIsInvalid,
    PreRegisterOwnerLockArgsIsInvalid,
    PreRegisterIsNotTimeout,
    PreRegisterRefundCapacityError,
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
    AccountCellInExpirationAuctionConfirmationPeriod = -115,
    AccountCellMissingPrevAccount = -114,
    AccountCellNextUpdateError,
    AccountCellStillCanNotRecycle,
    AccountCellIdNotMatch,
    AccountCellPermissionDenied = -110,
    AccountCellOwnerLockShouldNotBeModified,
    AccountCellOwnerLockShouldBeModified,
    AccountCellManagerLockShouldBeModified,
    AccountCellDataNotConsistent,
    AccountCellProtectFieldIsModified,
    AccountCellNoMoreFee,
    AccountCellInExpirationAuctionPeriod,
    AccountCellThrottle = -102,
    // ⚠️ DO NOT CHANGE
    AccountCellRenewDurationMustLongerThanYear,
    AccountCellRenewDurationBiggerThanPayed,
    // -100
    AccountCellInExpirationGracePeriod,
    AccountCellHasExpired,
    AccountCellIsNotExpired,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError, // -95
    AccountCellRecordKeyInvalid,
    AccountCellRecordSizeTooLarge,
    AccountCellRecordNotEmpty,
    AccountCellStatusLocked,
    EIP712SerializationError = -90,
    EIP712SematicError,
    EIP712DecodingWitnessArgsError,
    EIP712SignatureError,
    BalanceCellFoundSomeOutputsLackOfType = -80,
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
    ReverseRecordCellLockError = -60,
    ReverseRecordCellCapacityError,
    ReverseRecordCellAccountError,
    ReverseRecordCellChangeError,
    SubAccountFeatureNotEnabled = -50,
    SubAccountCellSMTRootError,
    SubAccountWitnessSMTRootError,
    SubAccountCellCapacityError,
    SubAccountCellAccountIdError,
    SubAccountCellConsistencyError,
    SubAccountInitialValueError,
    SubAccountSigVerifyError,
    SubAccountFieldNotEditable,
    SubAccountEditLockError,
    SubAccountJoinBetaError = -40,
    SubAccountProfitError,
    SubAccountCustomScriptError,
    SubAccountNormalCellLockLimit,
    SubAccountCollectProfitError,
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

pub trait ScriptError {
    fn as_i8(&self) -> i8;
}

#[derive(Debug, Clone)]
pub struct Error<T: Into<i8> + Copy> {
    pub code: T,
    pub message: String,
}

impl<T: Into<i8> + Copy> Error<T> {
    pub fn new(code: T, message: String) -> Self {
        Self { code, message }
    }

    pub fn boxed(code: T, message: String) -> Box<Self> {
        Box::new(Self { code, message })
    }
}

impl<T: Into<i8> + Copy> ScriptError for Error<T> {
    fn as_i8(&self) -> i8 {
        self.code.into()
    }
}

impl<T: Into<i8> + Copy> PartialEq for Error<T> {
    fn eq(&self, other: &Self) -> bool {
        self.code.into() == other.code.into()
    }
}

impl From<Error<ErrorCode>> for Box<dyn ScriptError> {
    fn from(err: Error<ErrorCode>) -> Box<dyn ScriptError> {
        Box::new(err)
    }
}

impl From<Box<Error<ErrorCode>>> for Box<dyn ScriptError> {
    fn from(err: Box<Error<ErrorCode>>) -> Box<dyn ScriptError> {
        err
    }
}

impl From<SysError> for Error<ErrorCode> {
    fn from(err: SysError) -> Self {
        Self::new(ErrorCode::from(err), String::new())
    }
}

impl From<SysError> for Box<dyn ScriptError> {
    fn from(err: SysError) -> Box<dyn ScriptError> {
        Box::new(Error::from(err))
    }
}

impl fmt::Debug for Box<dyn ScriptError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Box<dyn ScriptError>")
            .field("code", &self.as_i8())
            .finish()
    }
}
