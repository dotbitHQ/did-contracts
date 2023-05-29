#[cfg(feature = "no_std")]
use alloc::{format, string::String};
#[cfg(feature = "std")]
use std::str::FromStr;

#[cfg(feature = "no_std")]
pub use blake2b_ref::{Blake2b, Blake2bBuilder};
#[cfg(feature = "std")]
pub use blake2b_rs::{Blake2b, Blake2bBuilder};
#[cfg(feature = "no_std")]
use das_types::{constants::*, packed, prelude::*};
#[cfg(feature = "std")]
use das_types_std::{constants::*, packed, prelude::*};
#[cfg(feature = "std")]
use serde_json;

use crate::error::ASTError;
use crate::types::*;

macro_rules! gen_json_to_uint_fn {
    ($name:ident, $u_type:ty) => {
        #[cfg(feature = "std")]
        pub fn $name(key: String, obj: &serde_json::Value) -> Result<$u_type, ASTError> {
            if let Some(val) = obj.as_u64() {
                if val > <$u_type>::MAX as u64 {
                    return Err(ASTError::JsonValueError {
                        key,
                        val: format!("stringify!($u_type), but got {}", val),
                    });
                }
                Ok(val as $u_type)
            } else if let Some(val) = obj.as_str() {
                // Support string format uint, for example 1_000_000_000
                let number_str = val.replace("_", "");
                match number_str.parse::<u64>() {
                    Ok(val) => {
                        if val > <$u_type>::MAX as u64 {
                            return Err(ASTError::JsonValueError {
                                key,
                                val: format!("stringify!($u_type), but got {}", val),
                            });
                        }
                        Ok(val as $u_type)
                    }
                    Err(_) => Err(ASTError::JsonValueError {
                        key,
                        val: String::from(stringify!($u_type)),
                    }),
                }
            } else {
                Err(ASTError::JsonValueError {
                    key,
                    val: String::from(stringify!($u_type)),
                })
            }
        }
    };
}

const CKB_HASH_LENGTH: usize = 32;
const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
const CKB_HASH_EMPTY: [u8; 32] = [0u8; 32];

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_LENGTH)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}

pub fn blake2b_256<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    if s.as_ref().is_empty() {
        return CKB_HASH_EMPTY;
    }

    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn hex_to_bytes(key: String, mut input: &str) -> Result<Vec<u8>, ASTError> {
    input = input.trim_start_matches("0x");
    hex::decode(input).map_err(|_| ASTError::ParseHexFailed { key: key })
}

pub fn bytes_to_string(key: String, bytes: &[u8]) -> Result<String, ASTError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| ASTError::ParseUtf8StringFailed { key: key })
}

pub fn bytes_reader_to_string(key: String, reader: packed::BytesReader) -> Result<String, ASTError> {
    bytes_to_string(key, reader.raw_data())
}

gen_json_to_uint_fn!(json_to_u8, u8);
gen_json_to_uint_fn!(json_to_u32, u32);
gen_json_to_uint_fn!(json_to_u64, u64);

#[cfg(feature = "std")]
pub fn json_to_string(key: String, obj: &serde_json::Value) -> Result<String, ASTError> {
    match obj.as_str() {
        Some(str) => Ok(str.to_string()),
        None => Err(ASTError::JsonValueError {
            key,
            val: String::from("some string"),
        }),
    }
}

pub fn byte_to_symbol(key: String, byte: u8) -> Result<SymbolType, ASTError> {
    SymbolType::try_from(byte).map_err(|_| ASTError::UndefinedOperator { key: key, type_: byte })
}

pub fn byte_reader_to_symbol(key: String, reader: packed::ByteReader) -> Result<SymbolType, ASTError> {
    byte_to_symbol(key, reader.as_slice()[0])
}

#[cfg(feature = "std")]
pub fn json_to_symbol(key: String, obj: &serde_json::Value) -> Result<SymbolType, ASTError> {
    match obj.as_str() {
        Some(str) => {
            let symbol = SymbolType::from_str(str).map_err(|_| ASTError::JsonValueIsUndefined {
                key,
                val: str.to_string(),
            })?;
            Ok(symbol)
        }
        None => Err(ASTError::JsonValueError {
            key,
            val: String::from("some string"),
        }),
    }
}

pub fn mol_reader_to_operator(key: String, reader: packed::ASTOperatorReader) -> Result<OperatorExpression, ASTError> {
    let symbol = byte_reader_to_symbol(key.clone() + ".symbol", reader.symbol())?;
    let mut expressions = vec![];
    for (i, expr_reader) in reader.expressions().iter().enumerate() {
        let expr = mol_reader_to_expression(format!("{}.expressions[{}]", &key, i), expr_reader)?;
        expressions.push(expr);
    }

    Ok(OperatorExpression { symbol, expressions })
}

#[cfg(feature = "std")]
pub fn json_to_operator(key: String, obj: &serde_json::Value) -> Result<OperatorExpression, ASTError> {
    if obj["type"].as_str() != Some("operator") {
        return Err(ASTError::JsonValueError {
            key: key + ".type",
            val: String::from("operator"),
        });
    }

    let symbol = json_to_symbol(key.clone() + ".symbol", &obj["symbol"])?;

    match &obj["expressions"].as_array() {
        Some(arr) => {
            let mut expressions = vec![];
            for (i, expr) in arr.iter().enumerate() {
                let expr = json_to_expression(format!("{}.expressions[{}]", &key, i), expr)?;
                expressions.push(expr);
            }
            Ok(OperatorExpression { symbol, expressions })
        }
        None => Err(ASTError::JsonValueError {
            key: key + ".expressions",
            val: String::from("array"),
        }),
    }
}

pub fn byte_to_function_name(key: String, byte: u8) -> Result<FnName, ASTError> {
    FnName::try_from(byte).map_err(|_| ASTError::UndefinedOperator { key: key, type_: byte })
}

pub fn byte_reader_to_function_name(key: String, reader: packed::ByteReader) -> Result<FnName, ASTError> {
    byte_to_function_name(key, reader.as_slice()[0])
}

#[cfg(feature = "std")]
pub fn json_to_function_name(key: String, obj: &serde_json::Value) -> Result<FnName, ASTError> {
    match obj.as_str() {
        Some(str) => {
            let fn_name = FnName::from_str(str).map_err(|_| ASTError::JsonValueIsUndefined {
                key,
                val: String::from(str),
            })?;
            Ok(fn_name)
        }
        None => Err(ASTError::JsonValueError {
            key,
            val: String::from("some string"),
        }),
    }
}

pub fn mol_reader_to_function(key: String, reader: packed::ASTFunctionReader) -> Result<FunctionExpression, ASTError> {
    let name = byte_reader_to_function_name(key.clone() + ".name", reader.name())?;
    let mut arguments = vec![];
    for (i, expr_reader) in reader.arguments().iter().enumerate() {
        let expr = mol_reader_to_expression(format!("{}.arguments[{}]", &key, i), expr_reader)?;
        arguments.push(expr);
    }

    Ok(FunctionExpression { name, arguments })
}

#[cfg(feature = "std")]
pub fn json_to_function(key: String, obj: &serde_json::Value) -> Result<FunctionExpression, ASTError> {
    if obj["type"].as_str() != Some("function") {
        return Err(ASTError::JsonValueError {
            key: key + ".type",
            val: String::from("function"),
        });
    }

    let fn_name = json_to_function_name(key.clone() + ".name", &obj["name"])?;
    match &obj["arguments"] {
        serde_json::Value::Array(arr) => {
            let mut arguments = vec![];
            for (i, expr) in arr.iter().enumerate() {
                let expr = json_to_expression(format!("{}.arguments[{}]", &key, i), expr)?;
                arguments.push(expr);
            }
            return Ok(FunctionExpression {
                name: fn_name,
                arguments,
            });
        }
        _ => {
            return Err(ASTError::JsonValueError {
                key: key + ".arguments",
                val: String::from("some array"),
            });
        }
    }
}

pub fn byte_to_variable_name(key: String, byte: u8) -> Result<VarName, ASTError> {
    VarName::try_from(byte).map_err(|_| ASTError::UndefinedVariableType { key: key, type_: byte })
}

pub fn byte_reader_to_variable_name(key: String, reader: packed::ByteReader) -> Result<VarName, ASTError> {
    byte_to_variable_name(key, reader.as_slice()[0])
}

#[cfg(feature = "std")]
pub fn json_to_variable_name(key: String, obj: &serde_json::Value) -> Result<VarName, ASTError> {
    match obj.as_str() {
        Some(str) => {
            let var_name = VarName::from_str(str).map_err(|_| ASTError::JsonValueIsUndefined {
                key,
                val: String::from(str),
            })?;
            Ok(var_name)
        }
        None => Err(ASTError::JsonValueError {
            key,
            val: String::from("some string"),
        }),
    }
}

pub fn mol_reader_to_variable(key: String, reader: packed::ASTVariableReader) -> Result<VariableExpression, ASTError> {
    let name = byte_reader_to_variable_name(key.clone() + ".name", reader.name())?;
    Ok(VariableExpression { name })
}

#[cfg(feature = "std")]
pub fn json_to_variable(key: String, obj: &serde_json::Value) -> Result<VariableExpression, ASTError> {
    if obj["type"].as_str() != Some("variable") {
        return Err(ASTError::JsonValueError {
            key: key + ".type",
            val: String::from("variable"),
        });
    }

    let name = json_to_variable_name(key + ".name", &obj["name"])?;
    Ok(VariableExpression { name })
}

pub fn byte_to_value_type(key: String, byte: u8) -> Result<ValueType, ASTError> {
    ValueType::try_from(byte).map_err(|_| ASTError::UndefinedValueType { key: key, type_: byte })
}

pub fn byte_reader_to_value_type(key: String, reader: packed::ByteReader) -> Result<ValueType, ASTError> {
    byte_to_value_type(key, reader.as_slice()[0])
}

#[cfg(feature = "std")]
pub fn json_to_value_type(key: String, obj: &serde_json::Value) -> Result<ValueType, ASTError> {
    match obj.as_str() {
        Some(str) => Ok(ValueType::from_str(str).map_err(|_| ASTError::JsonValueIsUndefined {
            key,
            val: String::from(str),
        })?),
        None => {
            return Err(ASTError::JsonValueError {
                key,
                val: String::from("some string"),
            });
        }
    }
}

pub fn mol_reader_to_value(key: String, reader: packed::ASTValueReader) -> Result<ValueExpression, ASTError> {
    let extended_key = key.clone() + ".value";

    let value_type = byte_reader_to_value_type(key.clone() + ".value_type", reader.value_type())?;
    let value =
        match value_type {
            ValueType::Bool => {
                let bytes = reader.value().raw_data();
                Value::Bool(bytes[0] == 1)
            }
            ValueType::Uint8 => {
                let bytes = reader.value().raw_data();
                Value::Uint8(bytes[0])
            }
            ValueType::Uint32 => {
                let bytes = reader.value().raw_data();
                let num = u32::from_le_bytes(
                    bytes
                        .try_into()
                        .map_err(|_| ASTError::BytesToUint32Failed { key: extended_key })?,
                );
                Value::Uint32(num)
            }
            ValueType::Uint64 => {
                let bytes = reader.value().raw_data();
                let num = u64::from_le_bytes(
                    bytes
                        .try_into()
                        .map_err(|_| ASTError::BytesToUint64Failed { key: extended_key })?,
                );
                Value::Uint64(num)
            }
            ValueType::Binary => {
                let bytes = reader.value().raw_data();
                Value::Binary(bytes.to_vec())
            }
            ValueType::BinaryVec => {
                let bytes_vec_reader = packed::BytesVecReader::from_compatible_slice(reader.value().raw_data())
                    .map_err(|_| ASTError::BytesToEntityFailed { key: extended_key })?;
                let binary_vec = bytes_vec_reader.iter().map(|item| item.raw_data().to_vec()).collect();

                Value::BinaryVec(binary_vec)
            }
            ValueType::String => {
                let text = String::from_utf8(reader.value().raw_data().to_vec())
                    .map_err(|_| ASTError::ParseUtf8StringFailed { key: extended_key })?;

                Value::String(text)
            }
            ValueType::StringVec => {
                let bytes_vec_reader = packed::BytesVecReader::from_compatible_slice(reader.value().raw_data())
                    .map_err(|_| ASTError::BytesToEntityFailed {
                        key: extended_key.clone(),
                    })?;
                let mut text_vec = vec![];
                for item in bytes_vec_reader.iter() {
                    let text =
                        String::from_utf8(item.raw_data().to_vec()).map_err(|_| ASTError::ParseUtf8StringFailed {
                            key: extended_key.clone(),
                        })?;
                    text_vec.push(text);
                }

                Value::StringVec(text_vec)
            }
            ValueType::CharsetType => {
                let bytes = reader.value().raw_data();
                let num = u32::from_le_bytes(bytes.try_into().map_err(|_| ASTError::BytesToUint32Failed {
                    key: extended_key.clone(),
                })?);
                let charset = CharSetType::try_from(num).map_err(|_| ASTError::UndefinedCharSetType {
                    key: extended_key,
                    type_: num,
                })?;

                Value::CharsetType(charset)
            }
        };

    Ok(ValueExpression { value_type, value })
}

#[cfg(feature = "std")]
pub fn json_to_value(key: String, obj: &serde_json::Value) -> Result<ValueExpression, ASTError> {
    if obj["type"].as_str() != Some("value") {
        return Err(ASTError::JsonValueError {
            key: key + ".type",
            val: String::from("value"),
        });
    }

    let value_key = "value";
    let value_key_text = format!("{}.{}", key, value_key);
    let value_type = json_to_value_type(key.clone() + ".value_type", &obj["value_type"])?;
    let value = match value_type {
        ValueType::Bool => {
            let val = match obj[value_key].as_bool() {
                Some(val) => val,
                None => {
                    return Err(ASTError::JsonValueError {
                        key: format!("{}.{}", key, value_key),
                        val: String::from("some bool"),
                    });
                }
            };
            Value::Bool(val)
        }
        ValueType::Uint8 => Value::Uint8(json_to_u8(value_key_text, &obj[value_key])?),
        ValueType::Uint32 => Value::Uint32(json_to_u32(value_key_text, &obj[value_key])?),
        ValueType::Uint64 => Value::Uint64(json_to_u64(value_key_text, &obj[value_key])?),
        ValueType::Binary => {
            let val = match obj[value_key].as_str() {
                Some(val) => hex_to_bytes(value_key_text, &val)?,
                None => {
                    return Err(ASTError::JsonValueError {
                        key: value_key_text,
                        val: String::from("some string"),
                    });
                }
            };
            Value::Binary(val)
        }
        ValueType::BinaryVec => {
            let val = match obj[value_key].as_array() {
                Some(val) => {
                    let mut tmp = vec![];
                    for (i, v) in val.iter().enumerate() {
                        match v.as_str() {
                            Some(val) => {
                                let bytes = hex_to_bytes(format!("{}.{}[{}]", key, value_key, i), &val)?;
                                tmp.push(bytes);
                            }
                            None => {
                                return Err(ASTError::JsonValueError {
                                    key: format!("{}.{}.{}", key, value_key, i),
                                    val: String::from("some string"),
                                });
                            }
                        }
                    }
                    tmp
                }
                None => {
                    return Err(ASTError::JsonValueError {
                        key: value_key_text,
                        val: String::from("some string"),
                    });
                }
            };
            Value::BinaryVec(val)
        }
        ValueType::String => {
            let val = json_to_string(value_key_text, &obj[value_key])?;
            Value::String(val)
        }
        ValueType::StringVec => {
            let val = match obj[value_key].as_array() {
                Some(val) => {
                    let mut tmp = vec![];
                    for (i, v) in val.iter().enumerate() {
                        let val = json_to_string(format!("{}[{}]", value_key_text, i), v)?;
                        tmp.push(val)
                    }
                    tmp
                }
                None => {
                    return Err(ASTError::JsonValueError {
                        key: value_key_text,
                        val: String::from("some string"),
                    });
                }
            };

            Value::StringVec(val)
        }
        ValueType::CharsetType => {
            let val = match obj[value_key].as_str() {
                Some(val) => CharSetType::from_str(val).map_err(|_| ASTError::JsonValueIsUndefined {
                    key: value_key_text,
                    val: String::from(val),
                })?,
                None => {
                    return Err(ASTError::JsonValueError {
                        key: value_key_text,
                        val: String::from("some string"),
                    });
                }
            };

            Value::CharsetType(val)
        } // _ => todo!(),
    };

    Ok(ValueExpression { value_type, value })
}

pub fn byte_to_expression_type(key: String, byte: u8) -> Result<ExpressionType, ASTError> {
    ExpressionType::try_from(byte).map_err(|_| ASTError::UndefinedExpression { key: key, type_: byte })
}

pub fn byte_reader_to_expression_type(key: String, reader: packed::ByteReader) -> Result<ExpressionType, ASTError> {
    byte_to_expression_type(key, reader.as_slice()[0])
}

#[cfg(feature = "std")]
pub fn json_to_expression_type(key: String, obj: &serde_json::Value) -> Result<ExpressionType, ASTError> {
    match obj.as_str() {
        Some(str) => Ok(
            ExpressionType::from_str(str).map_err(|_| ASTError::JsonValueIsUndefined {
                key,
                val: String::from(str),
            })?,
        ),
        None => {
            return Err(ASTError::JsonValueError {
                key,
                val: String::from("some string"),
            });
        }
    }
}

pub fn mol_reader_to_expression(key: String, reader: packed::ASTExpressionReader) -> Result<Expression, ASTError> {
    let type_ = byte_reader_to_expression_type(key.clone() + ".type", reader.expression_type())?;

    match type_ {
        ExpressionType::Operator => {
            let reader = packed::ASTOperatorReader::from_compatible_slice(reader.expression().raw_data())
                .map_err(|_| ASTError::BytesToEntityFailed { key: key.clone() })?;

            Ok(Expression::Operator(mol_reader_to_operator(key.clone(), reader)?))
        }
        ExpressionType::Function => {
            let reader = packed::ASTFunctionReader::from_compatible_slice(reader.expression().raw_data())
                .map_err(|_| ASTError::BytesToEntityFailed { key: key.clone() })?;

            Ok(Expression::Function(mol_reader_to_function(key.clone(), reader)?))
        }
        ExpressionType::Variable => {
            let reader = packed::ASTVariableReader::from_compatible_slice(reader.expression().raw_data())
                .map_err(|_| ASTError::BytesToEntityFailed { key: key.clone() })?;

            Ok(Expression::Variable(mol_reader_to_variable(key.clone(), reader)?))
        }
        ExpressionType::Value => {
            let reader = packed::ASTValueReader::from_compatible_slice(reader.expression().raw_data())
                .map_err(|_| ASTError::BytesToEntityFailed { key: key.clone() })?;

            Ok(Expression::Value(mol_reader_to_value(key.clone(), reader)?))
        } // _ => todo!(),
    }
}

#[cfg(feature = "std")]
pub fn json_to_expression(key: String, obj: &serde_json::Value) -> Result<Expression, ASTError> {
    let type_ = json_to_expression_type(key.clone() + ".type", &obj["type"])?;

    let expr = match type_ {
        ExpressionType::Operator => Expression::Operator(json_to_operator(key, obj)?),
        ExpressionType::Function => Expression::Function(json_to_function(key, obj)?),
        ExpressionType::Variable => Expression::Variable(json_to_variable(key, obj)?),
        ExpressionType::Value => Expression::Value(json_to_value(key, obj)?),
        // _ => todo!(),
    };

    Ok(expr)
}

pub fn mol_reader_to_sub_account_rule(
    key: String,
    reader: packed::SubAccountRuleReader,
) -> Result<SubAccountRule, ASTError> {
    let name = bytes_reader_to_string(key.clone() + ".name", reader.name())?;
    let note = bytes_reader_to_string(key.clone() + ".note", reader.note())?;
    let status_int = u8::from(reader.status());
    let status = SubAccountRuleStatus::try_from(status_int).map_err(|_| ASTError::UndefinedRuleStatus {
        key: key.clone() + ".status",
        type_: status_int,
    })?;

    Ok(SubAccountRule {
        index: u32::from(reader.index()),
        name,
        note,
        price: u64::from(reader.price()),
        status,
        ast: mol_reader_to_expression(key + ".ast", reader.ast())?,
    })
}

pub fn mol_reader_to_sub_account_rules(
    key: String,
    reader: packed::SubAccountRulesReader,
) -> Result<Vec<SubAccountRule>, ASTError> {
    let mut tmp = vec![];
    for (i, reader) in reader.iter().enumerate() {
        tmp.push(mol_reader_to_sub_account_rule(format!("{}[{}]", key, i), reader)?);
    }

    Ok(tmp)
}

pub fn sub_account_rules_to_mol_entity(rules: Vec<SubAccountRule>) -> Result<packed::SubAccountRules, ASTError> {
    let mut tmp = vec![];
    for rule in rules.into_iter() {
        tmp.push(rule.into());
    }

    Ok(packed::SubAccountRules::new_builder().set(tmp).build())
}

#[cfg(feature = "std")]
pub fn json_to_sub_account_rule(key: String, obj: &serde_json::Value) -> Result<SubAccountRule, ASTError> {
    let index = json_to_u32(key.clone() + ".index", &obj["index"])?;
    let name = json_to_string(key.clone() + ".name", &obj["name"])?;
    let note = json_to_string(key.clone() + ".note", &obj["note"])?;
    let price = json_to_u64(key.clone() + ".price", &obj["price"])?;
    let status_int = json_to_u8(key.clone() + ".status", &obj["status"])?;
    let ast = json_to_expression(key.clone() + ".ast", &obj["ast"])?;

    let status = SubAccountRuleStatus::try_from(status_int).map_err(|_| ASTError::UndefinedRuleStatus {
        key: key + ".status",
        type_: status_int,
    })?;

    Ok(SubAccountRule {
        index,
        name,
        note,
        price,
        status,
        ast,
    })
}

#[cfg(feature = "std")]
pub fn json_to_sub_account_rules(key: String, obj: &serde_json::Value) -> Result<Vec<SubAccountRule>, ASTError> {
    match obj.as_array() {
        Some(arr) => {
            let mut tmp = vec![];
            for (i, obj) in arr.iter().enumerate() {
                tmp.push(json_to_sub_account_rule(format!("{}[{}]", key, i), obj)?);
            }

            Ok(tmp)
        }
        None => {
            return Err(ASTError::JsonValueError {
                key,
                val: String::from("array"),
            });
        }
    }
}
