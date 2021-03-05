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
    HeightCellIsRequired,
    QuoteCellIsRequired,
    ConfigIDIsUndefined,
    ConfigIsPartialMissing,
    ConfigCellIsRequired, // 15
    ConfigCellWitnessIsCorrupted,
    ConfigCellWitnessDecodingError,
    CellLockCanNotBeModified,
    CellTypeCanNotBeModified,
    CellDataCanNotBeModified, // 20
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
    PreRegisterQuoteIsInvalid = 70,
    ProposalFoundInvalidTransaction,
    ProposalMustIncludeSomePreAccountCell,
    ProposalSliceIsNotSorted,
    ProposalSliceIsDiscontinuity,
    ProposalSliceRelatedCellNotFound,
    ProposalCellTypeError,
    ProposalCellAccountIdError,
    ProposalFieldCanNotBeModified,
    ProposalWitnessCanNotBeModified,
    ProposalConfirmIdError = 80,
    ProposalConfirmNextError,
    ProposalConfirmExpiredAtError,
    ProposalConfirmAccountError,
    ProposalConfirmWitnessIDError,
    ProposalConfirmWitnessAccountError,
    ProposalConfirmWitnessOwnerError,
    ProposalConfirmWitnessManagerError,
    ProposalConfirmWitnessStatusError,
    ProposalConfirmWitnessRecordsError,
    ProposalConfirmWalletMissMatch = 90,
    ProposalConfirmWalletBalanceError,
    ProposalSliceNotEndCorrectly,
    ProposalSliceMustStartWithAccountCell,
    WalletFoundInvalidTransaction,
    WalletRequireAlwaysSuccess,
    WalletBaseCapacityIsWrong,
    WalletPermissionInvalid,
    PrevProposalItemNotFound,
    AccountCellFoundInvalidTransaction,
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
