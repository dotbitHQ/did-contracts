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
    HardCodedError,
    InvalidTransactionStructure,
    InvalidCellData,
    SuperLockIsRequired,
    CellMustUseSuperLock,
    TimeCellIsRequired,
    ConfigCellIsRequired, // 11
    WitnessReadingError = 20,
    WitnessEmpty,
    WitnessDasHeaderDecodingError,
    WitnessTypeDecodingError,
    WitnessActionIsNotTheFirst,
    WitnessActionDecodingError,
    WitnessEntityMissing,
    WitnessDataIsCorrupted,
    WitnessDataMissing,
    WitnessEntityDecodingError, // 29
    ActionNotSupported = 40,
    ConfigCellDataDecodingError,
    ApplyRegisterFoundInvalidTransaction,
    ApplyRegisterCellDataDecodingError,
    ApplyRegisterCellTimeError,
    ApplyRegisterNeedWaitLonger, // 45
    ApplyRegisterHasTimeout,
    PreRegisterFoundInvalidTransaction,
    PreRegisterApplyHashIsInvalid,
    PreRegisterCreateAtIsInvalid,
    PreRegisterAccountLengthMissMatch, // 50
    PreRegisterFoundUndefinedCharSet,
    PreRegisterCKBInsufficient,
    PreRegisterAccountCanNotRegisterNow,
    PreRegisterAccountCharSetConflict,
    PreRegisterAccountCharIsInvalid, // 55
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
