#[cfg(feature = "no_std")]
use alloc::string::String;

#[cfg(feature = "std")]
use thiserror::Error;
#[cfg(feature = "no_std")]
use thiserror_no_std::Error;

use crate::types::*;

#[derive(Error, Debug)]
pub enum ASTError {
    #[error("[{key}] Parsing hex string failed")]
    ParseHexFailed { key: String },
    #[error("[{key}] Parsing bytes to utf-8 string failed")]
    ParseUtf8StringFailed { key: String },
    #[error("[{key}] New molecule entity from bytes failed")]
    BytesToEntityFailed { key: String },
    #[error("[{key}] Parse bytes to uint32 failed")]
    BytesToUint32Failed { key: String },
    #[error("[{key}] Parse bytes to uint64 failed")]
    BytesToUint64Failed { key: String },
    #[error("[{key}] The rule status {type_} is undefined")]
    UndefinedRuleStatus { key: String, type_: u8 },
    #[error("[{key}] The charset type {type_} is undefined")]
    UndefinedCharSetType { key: String, type_: u32 },
    #[error("[{key}] The expression {type_} is undefined")]
    UndefinedExpression { key: String, type_: u8 },
    #[error("[{key}] The expression {type_} is unimplemented")]
    UnimplementedExpression { key: String, type_: ExpressionType },
    #[error("[{key}] The symbol {type_} is undefined")]
    UndefinedOperator { key: String, type_: u8 },
    #[error("[{key}] The function {type_} is undefined")]
    UndefinedFunction { key: String, type_: u8 },
    #[error("[{key}] The variable type {type_} is undefined")]
    UndefinedVariableType { key: String, type_: u8 },
    #[error("[{key}] The value type {type_} is undefined")]
    UndefinedValueType { key: String, type_: u8 },
    #[error("[{key}] The {key} should be {val}")]
    JsonValueError { key: String, val: String },
    #[error("[{key}] The {key} has an undefined value {val}")]
    JsonValueIsUndefined { key: String, val: String },
    #[error("[{key}] The param type should be {types}")]
    ParamTypeError { key: String, types: String },
    #[error("[{key}] The param type should be unique, but {types} found")]
    ParamTypeMismatch { key: String, types: String },
    #[error("[{key}] The length of the param should be {expected_length}, but {length}")]
    ParamLengthError {
        key: String,
        expected_length: String,
        length: String,
    },
    #[error("[{key}] The return type should be {types}")]
    ReturnTypeError { key: String, types: String },
    #[error("The values' type are mismatched")]
    ValueTypeMismatch,
    #[error("The value do not support this operator")]
    ValueOperatorUnsupported,
    #[error("[{key}] The expression must be a function or operator")]
    FunctionOrOperatorRequired { key: String },
}
