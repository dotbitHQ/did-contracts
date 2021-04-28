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
    ConfigIDIsUndefined,
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
    WitnessReadingError = 40,
    WitnessEmpty,
    WitnessDasHeaderDecodingError,
    WitnessTypeDecodingError,
    WitnessActionNotFound,
    WitnessActionDecodingError, // 45
    WitnessEntityMissing,
    WitnessDataParseLengthHeaderFailed,
    WitnessDataReadDataBodyFailed,
    WitnessDataDecodingError,
    WitnessDataHashMissMatch, // 50
    WitnessDataIndexMissMatch,
    WitnessEntityDecodingError,
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
    PreRegisterQuoteIsInvalid, // 80
    PreRegisterDiscountIsInvalid,
    PreRegisterOwnerLockArgsIsInvalid,
    ProposalFoundInvalidTransaction = 90,
    ProposalMustIncludeSomePreAccountCell,
    ProposalSliceIsNotSorted,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalSliceRelatedCellMissing, // 95
    ProposalCellTypeError,
    ProposalCellAccountIdError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmIdError, // 100
    ProposalConfirmNextError,
    ProposalConfirmExpiredAtError,
    ProposalConfirmAccountError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError, // 105
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError,
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalConfirmAccountLockArgsIsInvalid, // 110
    ProposalConfirmRefCellDuplicated,
    ProposalConfirmRefCellMissMatch,
    ProposalConfirmWalletMissMatch,
    ProposalConfirmWalletBalanceError,
    ProposalSliceNotEndCorrectly, // 115
    ProposalSliceMustStartWithAccountCell,
    ProposalSliceMustContainMoreThanOneElement,
    ProposalSliceItemMustBeUniqueAccount,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleCanNotFoundRefundCell,
    ProposalRecycleRefundAmountError,
    PrevProposalItemNotFound,
    WalletFoundInvalidTransaction = -126,
    WalletRequireAlwaysSuccess,
    WalletRequireWalletMakerLock,
    WalletRequireAccountId,
    WalletPermissionInvalid,
    WalletRefundError,
    AccountCellFoundInvalidTransaction = -110,
    AccountCellDataNotConsistent,
    AccountCellRefCellIsRequired,
    AccountCellOwnerCellIsRequired,
    AccountCellManagerCellIsRequired,
    AccountCellUnrelatedRefCellFound, // -105
    AccountCellRedundantRefCellNotAllowed,
    AccountCellProtectFieldIsModified,
    AccountCellRenewDurationMustLongerThanYear,
    AccountCellRenewDurationBiggerThanPaied,
    AccountCellInExpirationGracePeriod, // -100
    AccountCellHasExpired,
    AccountCellIsNotExpired,
    AccountCellRecycleCapacityError,
    AccountCellChangeCapacityError, // -96
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
