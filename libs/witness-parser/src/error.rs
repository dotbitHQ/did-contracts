#[cfg(feature = "no_std")]
use alloc::string::String;

#[cfg(feature = "no_std")]
use ckb_std::syscalls::SysError;
use das_types::constants::{DataType, Source};
#[cfg(feature = "std")]
use thiserror::Error;
#[cfg(feature = "no_std")]
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum WitnessParserError {
    #[error("witnesses[{index}] Occur an unreachable error.")]
    Unreachable { index: usize },
    #[error("The WitnessParser is not inited.")]
    InitializationRequired,
    #[error("witnesses[{index}] SysError: {err:?}")]
    SysError { index: usize, err: SysError },
    #[error("witnesses[{index}] Do not exist")]
    NotFoundByIndex { index: usize },
    #[error("witnesses[{index}] {msg}")]
    OrderError { index: usize, msg: String },
    #[error("witnesses[{index}] {msg}")]
    BasicDataStructureError { index: usize, msg: String },
    #[error("witnesses[{index}] The DataType {date_type} is undefined.")]
    UndefinedDataType { index: usize, date_type: u32 },
    #[error("witnesses[{index}] The witness of {data_type} found, but the cell is missing.")]
    ConfigCellNotFound { index: usize, data_type: DataType },
    #[error("witnesses[{index}] There should be only one {data_type} .")]
    DuplicatedConfigCellFound { index: usize, data_type: DataType },
    #[error("witnesses[{index}] Failed to load the bytes of Data from witness.")]
    LoadActionDataBodyFailed { index: usize },
    #[error("witnesses[{index}] Failed to decode the bytes of ActionData: {err}")]
    DecodingActionDataFailed { index: usize, err: String },
    #[error("witnesses[{index}] Failed to decode the ActionParams.")]
    DecodingActionParamsFailed { index: usize },
    #[error("witnesses[{index}] Failed to load the bytes of Data from witness.")]
    LoadDataBodyFailed { index: usize },
    #[error("witnesses[{index}] Failed to decode the bytes of Data: {err}")]
    DecodingDataFailed { index: usize, err: String },
    #[error("witnesses[{index}] Failed to get the verification hash from related cell: {msg}")]
    CanNotGetVerficicationHashFromCell { index: usize, msg: String },
    #[error("Failed to find the witness at witnesses[{index}]")]
    CanNotFindWitnessByIndex { index: usize },
    #[error("Failed to find the witness by {source:?}[{index}]")]
    CanNotFindWitnessByCellMeta { source: Source, index: usize },
    #[error("Failed to find the witness by {data_type}")]
    CanNotFindWitnessByDataType { data_type: DataType },
    #[error("witnesses[{index}] Failed to decode the bytes of Entity, the expected type is {data_type} v{version}.")]
    DecodingEntityFailed {
        index: usize,
        data_type: DataType,
        version: u32,
    },
}
