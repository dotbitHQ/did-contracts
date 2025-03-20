use alloc::string::ToString;

use ckb_std::syscalls::SysError;
use config::error::ConfigError;

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ckb_std::syscalls::debug(alloc::format!($($arg)*));
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
pub enum ErrorCode {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidTransactionStructure = 5,
    NotDidWitness = 6,
    InvalidDidCellWitnessHash = 7,
    ExpireAtNotEqual = 8,
    NotValidToDestroy = 9,
    WrongTypeArgs = 10,
    WrongClusterID = 11,
    WitnessParserError = 12,
    DifferentAction = 13,
    WrongCapacity = 14,
    ConfigError,
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

impl From<ConfigError> for ErrorCode {
    fn from(err: ConfigError) -> Self {
        warn!("ConfigError: {:?}", err.to_string());
        Self::ConfigError
    }
}

impl Into<i8> for ErrorCode {
    fn into(self) -> i8 {
        self as i8
    }
}
