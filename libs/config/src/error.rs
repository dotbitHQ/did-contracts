#[cfg(feature = "no_std")]
use alloc::string::String;

use das_types::constants::DataType;
#[cfg(feature = "std")]
use thiserror::Error;
#[cfg(feature = "no_std")]
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Decoding {name} from bytes to molecule entity failed.")]
    DecodingError { name: &'static str },
    #[error("Loading cell.type failed.")]
    LoadCellTypeError,
    #[error("cell_deps[{index}] Loading cell.data failed.")]
    LoadCellDataError { index: usize },
    #[error("cell_deps[{index}] Found a ConfigCell with undefined DataType({data_type}) .")]
    UndefinedDataType { index: usize, data_type: u32 },
    #[error("Can not found {data_type:?} in cell_deps.")]
    ConfigCellNotFound { data_type: DataType },
    #[error("The referred charset type {index} .")]
    UndefinedCharSetType { index: u32 },
    #[error("Can not find the field {expected_key} in ConfigCellMain.")]
    ConfigCellMainFieldMissing { expected_key: String },
    #[error("Found an undefined key {key} in ConfigCellMain.")]
    ConfigCellMainFieldUndefined { key: u32 },
    #[error("Can not decode the field {key} in ConfigCellMain.")]
    ConfigCellMainFieldDecodingError { key: &'static str },
}
