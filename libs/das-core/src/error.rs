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
    AccountIsReserved,
    WitnessReadingError = 30,
    WitnessEmpty,
    WitnessDasHeaderDecodingError,
    WitnessTypeDecodingError,
    WitnessActionIsNotTheFirst,
    WitnessActionDecodingError, // 35
    WitnessEntityMissing,
    WitnessDataDecodingError,
    WitnessDataIsCorrupted,
    WitnessDataMissing,
    WitnessEntityDecodingError,
    ApplyRegisterFoundInvalidTransaction = 50,
    ApplyRegisterCellDataDecodingError,
    ApplyRegisterCellHeightInvalid,
    ApplyRegisterNeedWaitLonger,
    ApplyRegisterHasTimeout,
    PreRegisterFoundInvalidTransaction = 60,
    PreRegisterAccountIdIsInvalid,
    PreRegisterApplyHashIsInvalid,
    PreRegisterCreateAtIsInvalid,
    PreRegisterAccountLengthMissMatch,
    PreRegisterFoundUndefinedCharSet,
    PreRegisterCKBInsufficient,
    PreRegisterAccountCanNotRegisterNow,
    PreRegisterAccountCharSetConflict,
    PreRegisterAccountCharIsInvalid,
    PreRegisterQuoteIsInvalid, // 70
    ProposalFoundInvalidTransaction = 80,
    ProposalMustIncludeSomePreAccountCell,
    ProposalSliceIsNotSorted,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalCellTypeError,
    ProposalCellAccountIdError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmIdError,
    ProposalConfirmNextError, // 90
    ProposalConfirmExpiredAtError,
    ProposalConfirmAccountError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError,
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError,
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalConfirmWalletMissMatch,
    ProposalConfirmWalletBalanceError, // 100
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell,
    ProposalRecycleNeedWaitLonger,
    ProposalRecycleCanNotFoundRefundCell,
    ProposalRecycleRefundAmountError,
    WalletFoundInvalidTransaction = 110,
    WalletRequireAlwaysSuccess,
    WalletPermissionInvalid,
    PrevProposalItemNotFound,
    AccountCellFoundInvalidTransaction = 120,
    AccountCellDataNotConsistent,
    AccountCellRefCellIsRequired,
    AccountCellOwnerCellIsRequired,
    AccountCellManagerCellIsRequired,
    AccountCellUnrelatedRefCellFound, // 125
    AccountCellRedundantRefCellNotAllowed,
    AccountCellProtectFieldIsModified,
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
