use ckb_std::error::SysError;

/// Error
#[derive(Debug)]
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Customized errors:
    HardCodedError, // 5
    InvalidTransactionStructure,
    InvalidCellData,
    InitDayHasPassed,
    TimeCellIsRequired = 10,
    TimeCellDataDecodingError,
    HeightCellIsRequired,
    HeightCellDataDecodingError,
    QuoteCellIsRequired,
    ConfigTypeIsUndefined,
    ConfigIsPartialMissing,
    ConfigCellIsRequired,
    ConfigCellWitnessIsCorrupted,
    ConfigCellWitnessDecodingError,
    CellLockCanNotBeModified = 20,
    CellTypeCanNotBeModified,
    CellDataCanNotBeModified,
    CellCapacityMustReduced,
    CellCapacityMustIncreased,
    CellCapacityMustConsistent, // 25
    CellsMustHaveSameOrderAndNumber,
    ActionNotSupported,
    SuperLockIsRequired,
    CellMustUseSuperLock,
    AlwaysSuccessLockIsRequired, // 30
    SignallLockIsRequired,
    AccountIsPreserved,
    DataTypeUpgradeRequired,
    NarrowMixerTypeFailed,
    AccountStillCanNotBeRegister,
    WitnessStructureError = 40,
    WitnessEmpty,
    WitnessDataTypeDecodingError,
    WitnessReadingError,
    WitnessActionDecodingError,
    WitnessDataParseLengthHeaderFailed, // 45
    WitnessDataReadDataBodyFailed,
    WitnessDataDecodingError,
    WitnessDataHashMissMatch,
    WitnessDataIndexMissMatch,
    WitnessEntityDecodingError, // 50
    ApplyRegisterFoundInvalidTransaction = 60,
    ApplyRegisterCellDataDecodingError,
    ApplyRegisterCellHeightInvalid,
    ApplyRegisterNeedWaitLonger,
    ApplyRegisterHasTimeout,
    PreRegisterFoundInvalidTransaction = 70,
    PreRegisterAccountIdIsInvalid,
    PreRegisterApplyHashIsInvalid,
    PreRegisterCreateAtIsInvalid,
    PreRegisterPriceInvalid,
    PreRegisterFoundUndefinedCharSet, // 75
    PreRegisterCKBInsufficient,
    PreRegisterAccountIsTooLong,
    PreRegisterAccountCharSetConflict,
    PreRegisterAccountCharIsInvalid,
    PreRegisterQuoteIsInvalid, // 80
    PreRegisterDiscountIsInvalid,
    PreRegisterOwnerLockArgsIsInvalid,
    ProposalFoundInvalidTransaction = 90,
    ProposalSliceIsNotSorted,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalSliceRelatedCellMissing,
    ProposalCellTypeError, // 95
    ProposalCellAccountIdError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmNewAccountCellDataError = 100,
    ProposalConfirmNewAccountCellCapacityError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError,
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError, // 105
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalConfirmAccountLockArgsIsInvalid = 110,
    ProposalConfirmIncomeError,
    ProposalConfirmRefundError,
    ProposalSlicesCanNotBeEmpty,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell, // 115
    ProposalSliceMustContainMoreThanOneElement,
    ProposalSliceItemMustBeUniqueAccount,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleRefundAmountError, // 120
    PrevProposalItemNotFound,
    IncomeCellConsolidateConditionNotSatisfied = -126,
    IncomeCellConsolidateError,
    IncomeCellConsolidateWaste,
    IncomeCellTransferError,
    IncomeCellCapacityError,
    AccountCellFoundInvalidTransaction = -110,
    AccountCellPermissionDenied,
    AccountCellOwnerLockShouldNotBeModified,
    AccountCellOwnerLockShouldBeModified,
    AccountCellManagerLockShouldBeModified,
    AccountCellDataNotConsistent,
    AccountCellProtectFieldIsModified,
    AccountCellNoMoreFee,
    AccountCellThrottle,
    AccountCellRenewDurationMustLongerThanYear,
    AccountCellRenewDurationBiggerThanPayed, // -100
    AccountCellInExpirationGracePeriod,
    AccountCellHasExpired,
    AccountCellIsNotExpired,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError, // -95
    AccountCellRecordKeyInvalid,
    AccountCellRecordSizeTooLarge,
    SystemOff = -1,
}

impl From<SysError> for Error {
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
