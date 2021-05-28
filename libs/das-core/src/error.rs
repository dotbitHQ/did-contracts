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
    SuperLockIsRequired,
    CellMustUseSuperLock,
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
    CellCapacityMustConsistent,
    CellsMustHaveSameOrderAndNumber,
    ActionNotSupported,
    AccountIsReserved, // 28
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
    PreRegisterAccountCharSetConflict,
    PreRegisterAccountCharIsInvalid,
    PreRegisterQuoteIsInvalid,
    PreRegisterDiscountIsInvalid = 80,
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
    ProposalConfirmIdError = 100,
    ProposalConfirmNextError,
    ProposalConfirmExpiredAtError,
    ProposalConfirmAccountError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError, // 105
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError,
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalConfirmAccountLockArgsIsInvalid = 110,
    ProposalConfirmIncomeError,
    ProposalConfirmRefundError,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell,
    ProposalSliceMustContainMoreThanOneElement, // 115
    ProposalSliceItemMustBeUniqueAccount,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleCanNotFoundRefundCell,
    ProposalRecycleRefundAmountError,
    PrevProposalItemNotFound, // 120
    IncomeCellInvalidTransaction = -126,
    IncomeCellConsolidateError,
    IncomeCellTransferError,
    AccountCellFoundInvalidTransaction = -110,
    AccountCellPermissionDenied,
    AccountCellOwnerLockShouldNotBeModified,
    AccountCellOwnerLockShouldBeModified,
    AccountCellManagerLockShouldBeModified,
    AccountCellDataNotConsistent,
    AccountCellProtectFieldIsModified,
    AccountCellRenewDurationMustLongerThanYear,
    AccountCellRenewDurationBiggerThanPaied,
    AccountCellInExpirationGracePeriod,
    AccountCellHasExpired, // -100
    AccountCellIsNotExpired,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError, // -97
    AccountCellRecordKeyInvalid,
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
