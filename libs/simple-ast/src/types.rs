#[cfg(feature = "no_std")]
use alloc::string::String;

#[cfg(feature = "no_std")]
use das_types::{constants::*, packed, prelude::*};
#[cfg(feature = "std")]
use das_types_std::{constants::*, packed, prelude::*};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "std")]
use serde::ser::{SerializeSeq, SerializeStruct};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize, Serializer};
use strum::{Display, EnumString};

use crate::error::ASTError;

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, Display)]
#[repr(u8)]
pub enum SubAccountRuleStatus {
    Off,
    On,
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct SubAccountRule {
    pub index: u32,
    pub name: String,
    pub note: String,
    pub price: u64,
    pub status: SubAccountRuleStatus,
    pub ast: Expression,
}

impl Into<packed::SubAccountRule> for SubAccountRule {
    fn into(self) -> packed::SubAccountRule {
        packed::SubAccountRuleBuilder::default()
            .index(packed::Uint32::from(self.index))
            .name(packed::Bytes::from(self.name.as_bytes()))
            .note(packed::Bytes::from(self.note.as_bytes()))
            .price(packed::Uint64::from(self.price))
            .status(packed::Uint8::from(self.status as u8))
            .ast(self.ast.into())
            .build()
    }
}

#[cfg(feature = "std")]
impl Serialize for SubAccountRule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SubAccountRule", 5)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("note", &self.note)?;

        if self.price > u32::MAX as u64 {
            state.serialize_field("price", &self.price.to_string())?;
        } else {
            state.serialize_field("price", &self.price)?;
        }

        state.serialize_field("status", &(self.status as u8))?;

        state.serialize_field("ast", &self.ast)?;
        state.end()
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum ExpressionType {
    Operator,
    Function,
    Variable,
    Value,
}

impl Into<packed::Byte> for ExpressionType {
    fn into(self) -> packed::Byte {
        packed::Byte::new(self as u8)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub enum Expression {
    Operator(OperatorExpression),
    Function(FunctionExpression),
    Variable(VariableExpression),
    Value(ValueExpression),
}

impl Into<packed::ASTExpression> for Expression {
    fn into(self) -> packed::ASTExpression {
        let (type_, mol_bytes) = match self {
            Expression::Operator(expr) => {
                let mol: packed::ASTOperator = expr.into();
                (ExpressionType::Operator, mol.as_slice().to_vec())
            }
            Expression::Function(expr) => {
                let mol: packed::ASTFunction = expr.into();
                (ExpressionType::Function, mol.as_slice().to_vec())
            }
            Expression::Variable(expr) => {
                let mol: packed::ASTVariable = expr.into();
                (ExpressionType::Variable, mol.as_slice().to_vec())
            }
            Expression::Value(expr) => {
                let mol: packed::ASTValue = expr.into();
                (ExpressionType::Value, mol.as_slice().to_vec())
            }
        };

        packed::ASTExpressionBuilder::default()
            .expression_type(type_.into())
            .expression(packed::Bytes::from(mol_bytes))
            .build()
    }
}

#[cfg(feature = "std")]
impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Expression::Operator(expr) => expr.serialize(serializer),
            Expression::Function(expr) => expr.serialize(serializer),
            Expression::Variable(expr) => expr.serialize(serializer),
            Expression::Value(expr) => expr.serialize(serializer),
        }
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumString, Display)]
#[repr(u8)]
pub enum SymbolType {
    #[cfg_attr(feature = "std", serde(rename(serialize = "not", deserialize = "not")))]
    #[strum(serialize = "not")]
    Not,
    #[cfg_attr(feature = "std", serde(rename(serialize = "and", deserialize = "and")))]
    #[strum(serialize = "and")]
    And,
    #[cfg_attr(feature = "std", serde(rename(serialize = "or", deserialize = "or")))]
    #[strum(serialize = "or")]
    Or,
    #[cfg_attr(feature = "std", serde(rename(serialize = ">", deserialize = ">")))]
    #[strum(serialize = ">")]
    Gt,
    #[cfg_attr(feature = "std", serde(rename(serialize = ">=", deserialize = ">=")))]
    #[strum(serialize = ">=")]
    Gte,
    #[cfg_attr(feature = "std", serde(rename(serialize = "<", deserialize = "<")))]
    #[strum(serialize = "<")]
    Lt,
    #[cfg_attr(feature = "std", serde(rename(serialize = "<=", deserialize = "<=")))]
    #[strum(serialize = "<=")]
    Lte,
    #[cfg_attr(feature = "std", serde(rename(serialize = "==", deserialize = "==")))]
    #[strum(serialize = "==")]
    Equal,
}

impl Into<packed::Byte> for SymbolType {
    fn into(self) -> packed::Byte {
        packed::Byte::new(self as u8)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct OperatorExpression {
    pub symbol: SymbolType,
    pub expressions: Vec<Expression>,
}

impl Into<packed::ASTOperator> for OperatorExpression {
    fn into(self) -> packed::ASTOperator {
        let expr_entities = packed::ASTExpressionsBuilder::default()
            .set(self.expressions.into_iter().map(Expression::into).collect())
            .build();

        packed::ASTOperatorBuilder::default()
            .symbol(self.symbol.into())
            .expressions(expr_entities)
            .build()
    }
}

#[cfg(feature = "std")]
impl Serialize for OperatorExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[cfg(feature = "std")]
        {
            let mut state = serializer.serialize_struct("OperatorExpression", 3)?;
            state.serialize_field("type", "operator")?;
            state.serialize_field("symbol", &self.symbol)?;
            state.serialize_field("expressions", &self.expressions)?;
            state.end()
        }
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumString, Display)]
#[cfg_attr(feature = "std", serde(rename_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum FnName {
    IncludeChars,
    IncludeWords,
    OnlyIncludeCharset,
    InList,
}

impl Into<packed::Byte> for FnName {
    fn into(self) -> packed::Byte {
        packed::Byte::new(self as u8)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct FunctionExpression {
    pub name: FnName,
    pub arguments: Vec<Expression>,
}

impl Into<packed::ASTFunction> for FunctionExpression {
    fn into(self) -> packed::ASTFunction {
        let expr_entities = packed::ASTExpressionsBuilder::default()
            .set(self.arguments.into_iter().map(Expression::into).collect())
            .build();

        packed::ASTFunctionBuilder::default()
            .name(self.name.into())
            .arguments(expr_entities)
            .build()
    }
}

#[cfg(feature = "std")]
impl Serialize for FunctionExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FunctionExpression", 3)?;
        state.serialize_field("type", "function")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("arguments", &self.arguments)?;
        state.end()
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, EnumString, Display)]
#[cfg_attr(feature = "std", serde(rename_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
#[repr(u8)]
pub enum VarName {
    Account,
    AccountChars,
    AccountLength,
}

impl Into<packed::Byte> for VarName {
    fn into(self) -> packed::Byte {
        packed::Byte::new(self as u8)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct VariableExpression {
    pub name: VarName,
}

impl Into<packed::ASTVariable> for VariableExpression {
    fn into(self) -> packed::ASTVariable {
        packed::ASTVariableBuilder::default().name(self.name.into()).build()
    }
}

#[cfg(feature = "std")]
impl Serialize for VariableExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("VariableExpression", 2)?;
        state.serialize_field("type", "variable")?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, IntoPrimitive, TryFromPrimitive, Display, EnumString)]
#[cfg_attr(feature = "std", serde(rename_all = "snake_case"))]
#[repr(u8)]
#[strum(serialize_all = "snake_case")]
pub enum ValueType {
    Bool,
    Uint8,
    Uint32,
    Uint64,
    Binary,
    #[cfg_attr(feature = "std", serde(rename(serialize = "binary[]", deserialize = "binary[]")))]
    #[strum(serialize = "binary[]")]
    BinaryVec,
    String,
    #[cfg_attr(feature = "std", serde(rename(serialize = "string[]", deserialize = "string[]")))]
    #[strum(serialize = "string[]")]
    StringVec,
    CharsetType,
}

impl Into<packed::Byte> for ValueType {
    fn into(self) -> packed::Byte {
        packed::Byte::new(self as u8)
    }
}

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct ValueExpression {
    pub value_type: ValueType,
    pub value: Value,
}

impl Into<packed::ASTValue> for ValueExpression {
    fn into(self) -> packed::ASTValue {
        packed::ASTValueBuilder::default()
            .value_type(self.value_type.into())
            .value(self.value.into())
            .build()
    }
}

#[cfg(feature = "std")]
impl Serialize for ValueExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ValueExpression", 3)?;
        state.serialize_field("type", "value")?;
        state.serialize_field("value_type", &self.value_type)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

pub type Binary = Vec<u8>;

#[cfg_attr(feature = "std", derive(Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Uint8(u8),
    Uint32(u32),
    Uint64(u64),
    Binary(Binary),
    BinaryVec(Vec<Binary>),
    String(String),
    StringVec(Vec<String>),
    CharsetType(CharSetType),
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Uint8(_) => ValueType::Uint8,
            Value::Uint32(_) => ValueType::Uint32,
            Value::Uint64(_) => ValueType::Uint64,
            Value::Binary(_) => ValueType::Binary,
            Value::BinaryVec(_) => ValueType::BinaryVec,
            Value::String(_) => ValueType::String,
            Value::StringVec(_) => ValueType::StringVec,
            Value::CharsetType(_) => ValueType::CharsetType,
        }
    }

    pub fn equal(&self, to: &Value) -> Result<bool, ASTError> {
        if self.get_type() != to.get_type() {
            return Err(ASTError::ValueTypeMismatch);
        }

        match (self, to) {
            (Value::Bool(val1), Value::Bool(val2)) => Ok(val1 == val2),
            (Value::Uint8(val1), Value::Uint8(val2)) => Ok(val1 == val2),
            (Value::Uint32(val1), Value::Uint32(val2)) => Ok(val1 == val2),
            (Value::Uint64(val1), Value::Uint64(val2)) => Ok(val1 == val2),
            (Value::Binary(val1), Value::Binary(val2)) => Ok(val1 == val2),
            (Value::BinaryVec(val1), Value::BinaryVec(val2)) => Ok(val1 == val2),
            (Value::String(val1), Value::String(val2)) => Ok(val1 == val2),
            (Value::StringVec(val1), Value::StringVec(val2)) => Ok(val1 == val2),
            (Value::CharsetType(val1), Value::CharsetType(val2)) => Ok(val1 == val2),
            _ => Err(ASTError::ValueOperatorUnsupported),
        }
    }

    pub fn compare(&self, right: &Value, symbol_type: SymbolType) -> Result<bool, ASTError> {
        let left = self.get_u64();
        let right = right.get_u64();

        match (left, right) {
            (Ok(left), Ok(right)) => {
                match symbol_type {
                    SymbolType::Gt => Ok(left > right),
                    SymbolType::Gte => Ok(left >= right),
                    SymbolType::Lt => Ok(left < right),
                    SymbolType::Lte => Ok(left <= right),
                    SymbolType::Equal => Ok(left == right),
                    _ => Err(ASTError::ValueOperatorUnsupported),
                }
            },
            _ => Err(ASTError::ValueOperatorUnsupported),
        }
    }

    fn get_u64(&self) -> Result<u64, ASTError> {
        match self {
            Value::Uint8(val) => Ok(*val as u64),
            Value::Uint32(val) => Ok(*val as u64),
            Value::Uint64(val) => Ok(*val),
            _ => Err(ASTError::ValueOperatorUnsupported),
        }
    }
}

impl Into<packed::Bytes> for Value {
    fn into(self) -> packed::Bytes {
        match self {
            Value::Bool(val) => packed::Bytes::from(if val { vec![1] } else { vec![0] }),
            Value::Uint8(val) => packed::Bytes::from(val.to_le_bytes().as_slice()),
            Value::Uint32(val) => packed::Bytes::from(val.to_le_bytes().as_slice()),
            Value::Uint64(val) => packed::Bytes::from(val.to_le_bytes().as_slice()),
            Value::Binary(val) => packed::Bytes::from(val),
            Value::BinaryVec(val) => {
                let bytes_vec = val.into_iter().map(|item| packed::Bytes::from(item)).collect();
                let bytes_vec_entity = packed::BytesVecBuilder::default().set(bytes_vec).build();

                packed::Bytes::from(bytes_vec_entity.as_slice())
            }
            Value::String(val) => packed::Bytes::from(val.as_bytes()),
            Value::StringVec(val) => {
                let bytes_vec = val
                    .into_iter()
                    .map(|item| packed::Bytes::from(item.as_bytes()))
                    .collect();
                let bytes_vec_entity = packed::BytesVecBuilder::default().set(bytes_vec).build();

                packed::Bytes::from(bytes_vec_entity.as_slice())
            }
            Value::CharsetType(val) => packed::Bytes::from((val as u32).to_le_bytes().as_slice()),
        }
    }
}

#[cfg(feature = "std")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(val) => serializer.serialize_bool(*val),
            Value::Uint8(val) => serializer.serialize_u8(*val),
            Value::Uint32(val) => serializer.serialize_u32(*val),
            Value::Uint64(val) => {
                if *val > u32::MAX as u64 {
                    serializer.serialize_str(&val.to_string())
                } else {
                    serializer.serialize_u64(*val)
                }
            }
            Value::Binary(val) => {
                let hex = hex::encode(val);
                serializer.serialize_str(&format!("0x{}", hex))
            }
            Value::BinaryVec(val) => {
                let mut seq = serializer.serialize_seq(Some(val.len()))?;
                for item in val {
                    seq.serialize_element(&format!("0x{}", hex::encode(item)))?;
                }
                seq.end()
            }
            Value::String(val) => serializer.serialize_str(val),
            Value::StringVec(val) => {
                let mut seq = serializer.serialize_seq(Some(val.len()))?;
                for item in val {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::CharsetType(val) => serializer.serialize_str(&val.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;
    use crate::util;

    #[test]
    fn test_value_from_to_mol() {
        let expected_bytes = "120000000c0000000d000000000100000001";
        let expected_expr = ValueExpression {
            value_type: ValueType::Bool,
            value: Value::Bool(true),
        };

        let mol: packed::ASTValue = expected_expr.clone().into();
        assert_eq!(expected_bytes, hex::encode(mol.as_slice()));

        let mol = packed::ASTValue::from_slice(&hex::decode(expected_bytes).unwrap()).unwrap();
        let value = util::mol_reader_to_value(String::from("."), mol.as_reader()).unwrap();
        assert!(matches!(
            value,
            ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            }
        ));
    }

    #[test]
    fn test_variable_from_to_mol() {
        let expected_bytes = "090000000800000000";
        let expected_expr = VariableExpression { name: VarName::Account };

        let mol: packed::ASTVariable = expected_expr.into();
        assert_eq!(expected_bytes, hex::encode(mol.as_slice()));

        let mol = packed::ASTVariable::from_slice(&hex::decode(expected_bytes).unwrap()).unwrap();
        let value = util::mol_reader_to_variable(String::from("."), mol.as_reader()).unwrap();
        assert!(matches!(value, VariableExpression { name: VarName::Account }));
    }

    #[test]
    fn test_function_from_to_mol() {
        let expected_bytes = "590000000c0000000d000000024c0000000c000000260000001a0000000c0000000d0000000209000000090000000800000001260000000c0000000d0000000315000000150000000c0000000d000000080400000000000000";
        let expected_expr = FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::CharsetType,
                    value: Value::CharsetType(CharSetType::Emoji),
                }),
            ],
        };

        let mol: packed::ASTFunction = expected_expr.into();
        assert_eq!(expected_bytes, hex::encode(mol.as_slice()));

        let mol = packed::ASTFunction::from_slice(&hex::decode(expected_bytes).unwrap()).unwrap();
        let value = util::mol_reader_to_function(String::from("."), mol.as_reader()).unwrap();
        assert!(matches!(&value, FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: args,
        } if args.len() == 2));
        assert!(matches!(
            &value.arguments[0],
            Expression::Variable(VariableExpression {
                name: VarName::AccountChars,
            })
        ));
        assert!(matches!(
            &value.arguments[1],
            Expression::Value(ValueExpression {
                value_type: ValueType::CharsetType,
                value: Value::CharsetType(CharSetType::Emoji),
            })
        ));
    }

    #[test]
    fn test_operator_from_to_mol() {
        let expected_bytes = "5f0000000c0000000d00000001520000000c0000002f000000230000000c0000000d0000000312000000120000000c0000000d000000000100000001230000000c0000000d0000000312000000120000000c0000000d000000000100000001";
        let expected_expr = OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
            ],
        };

        let mol: packed::ASTOperator = expected_expr.into();
        assert_eq!(expected_bytes, hex::encode(mol.as_slice()));

        let mol = packed::ASTOperator::from_slice(&hex::decode(expected_bytes).unwrap()).unwrap();
        let value = util::mol_reader_to_operator(String::from("."), mol.as_reader()).unwrap();
        assert!(matches!(&value, OperatorExpression {
            symbol: SymbolType::And,
            expressions: args,
        } if args.len() == 2));
        assert!(matches!(
            &value.expressions[0],
            Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            })
        ));
        assert!(matches!(
            &value.expressions[1],
            Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            })
        ));
    }

    macro_rules! test_value_from_to_json {
        (
            $fn_name:ident,
            $value_type_json: expr,
            $value_json: expr,
            $value_type: expr,
            $value: expr,
            $value_pat: pat_param
        ) => {
            paste::paste! {
                #[test]
                fn [<test_ $fn_name _from_to_json>]() {
                    let expected_json = json!({
                        "type": "value",
                        "value_type": $value_type_json,
                        "value": $value_json,
                    });
                    let expected_expr = ValueExpression {
                        value_type: $value_type,
                        value: $value,
                    };

                    let value = util::json_to_value(String::new(), &expected_json).unwrap();
                    assert!(matches!(
                        value,
                        ValueExpression {
                            value_type: $value_type,
                            value: $value_pat,
                        }
                    ));

                    let json = serde_json::to_value(&expected_expr).unwrap();
                    assert_eq!(expected_json, json);
                }
            }
        };
        ($value_type_json: expr, $value_json: expr, $value_type: expr, $value: expr) => {
            paste::paste! {
                #[test]
                fn [<test_ $value_type_json _from_to_json>]() {
                    let expected_json = json!({
                        "type": "value",
                        "value_type": $value_type_json,
                        "value": $value_json,
                    });
                    let expected_expr = ValueExpression {
                        value_type: $value_type,
                        value: $value,
                    };

                    let value = util::json_to_value(String::new(), &expected_json).unwrap();
                    assert!(matches!(
                        value,
                        ValueExpression {
                            value_type: $value_type,
                            value: $value,
                        }
                    ));

                    let json = serde_json::to_value(&expected_expr).unwrap();
                    assert_eq!(expected_json, json);
                }
            }
        };
    }

    test_value_from_to_json!("bool", true, ValueType::Bool, Value::Bool(true));
    test_value_from_to_json!("uint8", u8::MAX, ValueType::Uint8, Value::Uint8(u8::MAX));
    test_value_from_to_json!("uint32", u32::MAX, ValueType::Uint32, Value::Uint32(u32::MAX));
    test_value_from_to_json!(
        "uint64",
        "100000000000",
        ValueType::Uint64,
        Value::Uint64(100_000_000_000u64)
    );
    test_value_from_to_json!(
        binary,
        "binary",
        "0x1234",
        ValueType::Binary,
        Value::Binary(vec![0x12, 0x34]),
        Value::Binary(_)
    );
    test_value_from_to_json!(
        binary_vec,
        "binary[]",
        ["0x1234", "0x5678"],
        ValueType::BinaryVec,
        Value::BinaryVec(vec![vec![0x12, 0x34], vec![0x56, 0x78]]),
        Value::BinaryVec(_)
    );
    test_value_from_to_json!(
        string,
        "string",
        "text",
        ValueType::String,
        Value::String(String::from("text")),
        Value::String(_)
    );
    test_value_from_to_json!(
        string_vec,
        "string[]",
        ["text1", "text2"],
        ValueType::StringVec,
        Value::StringVec(vec![String::from("text1"), String::from("text2")]),
        Value::StringVec(_)
    );

    #[test]
    fn test_variable_from_to_json() {
        let expected_json = json!({
            "type": "variable",
            "name": "account",
        });
        let expected_expr = VariableExpression { name: VarName::Account };

        let value = util::json_to_variable(String::new(), &expected_json).unwrap();
        assert!(matches!(value, VariableExpression { name: VarName::Account }));

        let json = serde_json::to_value(&expected_expr).unwrap();
        assert_eq!(expected_json, json);
    }

    #[test]
    fn test_function_from_to_json() {
        let expected_json = json!({
            "type": "function",
            "name": "only_include_charset",
            "arguments": [
                {
                    "type": "variable",
                    "name": "account_chars",
                },
                {
                    "type": "value",
                    "value_type": "charset_type",
                    "value": "Emoji",
                },
            ],
        });
        let _expected_expr = FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::CharsetType,
                    value: Value::CharsetType(CharSetType::Emoji),
                }),
            ],
        };

        let value = util::json_to_function(String::new(), &expected_json).unwrap();
        assert!(matches!(&value, FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: args,
        } if args.len() == 2));
        assert!(matches!(
            value.arguments.as_slice(),
            [
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::CharsetType,
                    value: Value::CharsetType(CharSetType::Emoji),
                }),
            ]
        ));

        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(expected_json, json);
    }

    #[test]
    fn test_operator_from_to_json() {
        let expected_json = json!({
            "type": "operator",
            "symbol": "and",
            "expressions": [
                {
                    "type": "value",
                    "value_type": "bool",
                    "value": true,
                },
                {
                    "type": "value",
                    "value_type": "bool",
                    "value": false,
                },
            ],
        });
        let expected_expr = OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                }),
            ],
        };

        let value = util::json_to_operator(String::new(), &expected_json).unwrap();
        assert!(matches!(&value, OperatorExpression {
            symbol: SymbolType::And,
            expressions: args,
        } if args.len() == 2));
        assert!(matches!(
            value.expressions.as_slice(),
            [
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                })
            ]
        ));

        let json = serde_json::to_value(&expected_expr).unwrap();
        assert_eq!(expected_json, json);
    }

    // #[test]
    // fn test_sub_account_rules_from_json() {
    //     let expected_json = json!([
    //         {
    //             "index": 0,
    //             "name": "Price of 1 Charactor Emoji DID",
    //             "note": "",
    //             "price": "",
    //             "ast": {
    //                 "type": "operator",
    //                 "symbol": "and",
    //                 "expressions": [
    //                     {
    //                         "type": "operator",
    //                         "symbol": "==",
    //                         "expressions": [
    //                             {
    //                                 "type": "variable",
    //                                 "name": "account_length",
    //                             },
    //                             {
    //                                 "type": "value",
    //                                 "value_type": "uint8",
    //                                 "value": 1,
    //                             },
    //                         ],
    //                     },
    //                     {
    //                         "type": "function",
    //                         "name": "only_include_charset",
    //                         "arguments": [
    //                             {
    //                                 "type": "variable",
    //                                 "name": "account_chars",
    //                             },
    //                             {
    //                                 "type": "value",
    //                                 "value_type": "charset_type",
    //                                 "value": "Emoji",
    //                             }
    //                         ],
    //                     }
    //                 ],
    //             }
    //         }
    //     ]);

    //     let _expected_expr =
    // }

    #[test]
    fn test_value_equal() {
        let val1 = Value::Uint8(u8::MAX);
        let val2 = Value::Uint8(u8::MAX);
        let val3 = Value::Uint8(u8::MIN);

        assert!(val1.equal(&val2).unwrap());
        assert!(!val1.equal(&val3).unwrap());
    }

    #[test]
    fn test_value_compare() {
        let mid = Value::Uint8(100);
        let left = Value::Uint32(99);
        let right = Value::Uint64(101);

        assert!(mid.compare(&mid, SymbolType::Equal).unwrap());
        assert!(mid.compare(&left, SymbolType::Gt).unwrap());
        assert!(mid.compare(&left, SymbolType::Gte).unwrap());
        assert!(mid.compare(&mid, SymbolType::Gte).unwrap());
        assert!(mid.compare(&right, SymbolType::Lt).unwrap());
        assert!(mid.compare(&right, SymbolType::Lte).unwrap());
        assert!(mid.compare(&mid, SymbolType::Lte).unwrap());
    }

    #[test]
    fn test_sub_account_rule_from_to_mol() {
        let expected_bytes = "5f0000001c000000200000002f000000330000003b0000005e0000000a0000000b0000003120e4bd8de8b4a6e688b700000000404b4c0000000000230000000c0000000d0000000312000000120000000c0000000d00000000010000000101";
        let expected_expr = SubAccountRule {
            index: 10,
            name: String::from("1 位账户"),
            note: String::from(""),
            price: 5_000_000,
            status: SubAccountRuleStatus::On,
            ast: Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            }),
        };

        let mol: packed::SubAccountRule = expected_expr.into();
        assert_eq!(expected_bytes, hex::encode(mol.as_slice()));

        let mol = packed::SubAccountRule::from_slice(&hex::decode(expected_bytes).unwrap()).unwrap();
        let rule = util::mol_reader_to_sub_account_rule(String::from("."), mol.as_reader()).unwrap();
        assert!(matches!(rule, SubAccountRule {
            index: 10,
            name,
            note,
            price: 5_000_000,
            status: SubAccountRuleStatus::On,
            ast: _,
        } if name == String::from("1 位账户") && note == String::new()));
    }
}
