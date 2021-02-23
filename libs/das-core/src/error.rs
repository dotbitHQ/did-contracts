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
    TimeCellIsRequired, // 10
    QuoteCellIsRequired,
    ConfigCellIsRequired,
    ConfigCellWitnessInvalid,
    CellLockCanNotBeModified,
    CellTypeCanNotBeModified, // 15
    CellDataCanNotBeModified,
    CellCapacityMustReduced,
    CellCapacityMustIncreased,
    CellCapacityMustConsistent,
    CellsMustHaveSameOrderAndNumber, // 20
    ActionNotSupported,
    WitnessReadingError = 30,
    WitnessEmpty,
    WitnessDasHeaderDecodingError,
    WitnessTypeDecodingError,
    WitnessActionIsNotTheFirst,
    WitnessActionDecodingError, // 35
    WitnessEntityMissing,
    WitnessDataIsCorrupted,
    WitnessDataMissing,
    WitnessEntityDecodingError,
    ApplyRegisterFoundInvalidTransaction = 50,
    ApplyRegisterCellDataDecodingError,
    ApplyRegisterCellTimeError,
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
    PreRegisterQuoteIsInvalid,
    ProposalFoundInvalidTransaction, // 70
    ProposalMustIncludeSomePreAccountCell,
    ProposalSliceIsNotSorted,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalCellTypeError,
    ProposalCellAccountIdError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmIdError,
    ProposalConfirmNextError, // 80
    ProposalConfirmExpiredAtError,
    ProposalConfirmAccountError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError,
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError,
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell, // 90
    WalletFoundInvalidTransaction,
    WalletRequireAlwaysSuccess,
    WalletBaseCapacityIsWrong,
    WalletPermissionInvalid,
    PrevProposalItemNotFound,
    TimeCellDataDecodingError = 100,
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
