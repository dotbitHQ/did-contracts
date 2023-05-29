#[cfg(feature = "no_std")]
use alloc::format;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::string::ToString;

#[cfg(feature = "no_std")]
use das_types::{constants::*, packed, prelude::*};
#[cfg(feature = "std")]
use das_types_std::{constants::*, packed, prelude::*};

use crate::error::ASTError;
use crate::types::*;
use crate::util::*;

fn assert_param_length(key: String, length: usize, expected_length: usize) -> Result<(), ASTError> {
    if length != expected_length {
        return Err(ASTError::ParamLengthError {
            key,
            expected_length: expected_length.to_string(),
            length: format!("it is {}", length),
        });
    }

    Ok(())
}

fn assert_param_length_gte(key: String, length: usize, expected_length: usize) -> Result<(), ASTError> {
    if length < expected_length {
        return Err(ASTError::ParamLengthError {
            key,
            expected_length: format!(">= {}", expected_length),
            length: format!("it is {}", length),
        });
    }

    Ok(())
}

macro_rules! assert_param_expression {
    ($key: expr, $param: expr, $expr: pat_param, $msg_types: expr) => {
        if !matches!($param, $expr) {
            return Err(ASTError::ParamTypeError {
                key: $key,
                types: $msg_types,
            });
        }
    };
}

macro_rules! assert_and_get_return {
    ($key: expr, $value: expr, $value_type: ident) => {
        match $value {
            Value::$value_type(val) => val,
            _ => {
                return Err(ASTError::ReturnTypeError {
                    key: $key,
                    types: ValueType::$value_type.to_string(),
                })
            }
        }
    };
}

pub fn match_rule_with_account_chars<'a>(
    rules: &'a [SubAccountRule],
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Option<&'a SubAccountRule>, ASTError> {
    for (i, rule) in rules.iter().enumerate() {
        if rule.status == SubAccountRuleStatus::Off {
            continue;
        }

        match rule.ast {
            Expression::Function(_) | Expression::Operator(_) => {}
            _ => {
                return Err(ASTError::FunctionOrOperatorRequired {
                    key: format!("rules[{}].ast", i),
                })
            }
        }

        let value = handle_expression(format!("rules[{}].ast", i), &rule.ast, account_chars, account)?;
        let ret = assert_and_get_return!(format!("rules[{}]", i), value, Bool);

        if ret {
            return Ok(Some(rule));
        }
    }

    Ok(None)
}

fn handle_expression(
    key: String,
    ast: &Expression,
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    let value = match ast {
        Expression::Operator(operator) => handle_operator(key, operator, account_chars, account)?,
        Expression::Function(function) => handle_function(key, function, account_chars, account)?,
        Expression::Variable(variable) => handle_variable(key, variable, account_chars, account)?,
        Expression::Value(value) => value.value.clone(),
        // _ => todo!()
    };

    Ok(value)
}

fn handle_operator(
    key: String,
    operator: &OperatorExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    let ret = match operator.symbol {
        SymbolType::And => operator_and_or(&key, operator, account_chars, account, true)?,
        SymbolType::Or => operator_and_or(&key, operator, account_chars, account, false)?,
        SymbolType::Not => operator_not(&key, operator, account_chars, account)?,
        SymbolType::Equal | SymbolType::Gt | SymbolType::Gte | SymbolType::Lt | SymbolType::Lte =>
            operator_compare(&key, operator, account_chars, account, operator.symbol)?,
        // _ => todo!(),
    };

    Ok(Value::Bool(ret))
}

fn operator_and_or(
    key: &str,
    operator: &OperatorExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
    is_and: bool,
) -> Result<bool, ASTError> {
    assert_param_length_gte(format!("{}.expressions", key), operator.expressions.len(), 2)?;

    let mut ret = if is_and { true } else { false };
    for (i, expression) in operator.expressions.iter().enumerate() {
        let value = handle_expression(
            format!("{}.expressions[{}]", key, i),
            expression,
            account_chars,
            account,
        )?;
        match value {
            Value::Bool(val) => {
                if is_and {
                    if !val {
                        ret = false;
                    }
                } else {
                    if val {
                        ret = true;
                    }
                }
            }
            _ => {
                return Err(ASTError::ParamTypeError {
                    key: format!("{}.expressions[{}]", key, i),
                    types: ValueType::Bool.to_string(),
                })
            }
        }
    }

    Ok(ret)
}

fn operator_not(
    key: &str,
    operator: &OperatorExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<bool, ASTError> {
    assert_param_length(format!("{}.expressions", key), operator.expressions.len(), 1)?;

    let value = handle_expression(
        format!("{}.expressions[0]", key),
        &operator.expressions[0],
        account_chars,
        account,
    )?;
    match value {
        Value::Bool(val) => Ok(!val),
        _ => {
            return Err(ASTError::ParamTypeError {
                key: format!("{}.expressions[0]", key),
                types: ValueType::Bool.to_string(),
            })
        }
    }
}

fn operator_compare(
    key: &str,
    operator: &OperatorExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
    symbol_type: SymbolType,
) -> Result<bool, ASTError> {
    assert_param_length(format!("{}.expressions", key), operator.expressions.len(), 2)?;

    let left = handle_expression(
        format!("{}.expressions[0]", key),
        &operator.expressions[0],
        account_chars,
        account,
    )?;
    let right = handle_expression(
        format!("{}.expressions[1]", key),
        &operator.expressions[1],
        account_chars,
        account,
    )?;

    if ![ValueType::Uint8, ValueType::Uint32, ValueType::Uint64].contains(&left.get_type()) {
        return Err(ASTError::ParamTypeError {
            key: format!("{}.expressions[0]", key),
            types: format!("Uint8, Uint32, Uint64"),
        });
    }

    left.compare(&right, symbol_type).map_err(|err| ASTError::ParamTypeError {
        key: key.to_string(),
        types: err.to_string(),
    })
}

fn handle_function(
    key: String,
    function: &FunctionExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    macro_rules! call_fn {
        ($fn_name: ident, $arg_len: expr) => {{
            assert_param_length(key.clone() + ".arguments", function.arguments.len(), $arg_len)?;
            $fn_name(key.clone(), &function.arguments, account_chars, account)
        }};
    }

    let ret = match function.name {
        FnName::IncludeChars | FnName::IncludeWords => call_fn!(include_chars, 2),
        FnName::OnlyIncludeCharset => call_fn!(only_include_charset, 2),
        FnName::InList => call_fn!(in_list, 2),
    }?;

    if ret.get_type() != ValueType::Bool {
        return Err(ASTError::ReturnTypeError {
            key,
            types: ValueType::Bool.to_string(),
        });
    }

    Ok(ret)
}

fn handle_variable(
    key: String,
    variable: &VariableExpression,
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    let ret = match variable.name {
        VarName::Account => Value::String(account.to_string()),
        VarName::AccountChars => {
            let mut string_vec = vec![];
            for (i, char) in account_chars.iter().enumerate() {
                let char = String::from_utf8(char.bytes().raw_data().to_owned()).map_err(|_| {
                    ASTError::ParseUtf8StringFailed {
                        key: format!("{}[{}]", key, i),
                    }
                })?;
                string_vec.push(char);
            }

            Value::StringVec(string_vec)
        }
        VarName::AccountLength => Value::Uint32(account_chars.len() as u32), // _ => todo!(),
    };

    Ok(ret)
}

fn include_chars(
    key: String,
    arguments: &[Expression],
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    assert_param_expression!(format!("{}.arguments[0]", key), arguments[0], Expression::Variable(VariableExpression {
        name: VarName::Account
    }), format!("variable AccountChars"));

    let chars = handle_expression(key.clone() + ".arguments[1]", &arguments[1], account_chars, account)?;

    assert_param_expression!(format!("{}.arguments[1]", key), chars, Value::StringVec(_), format!("string[]"));

    match chars {
        Value::StringVec(chars) => {
            for char in chars.iter() {
                if account.contains(char) {
                    return Ok(Value::Bool(true));
                }
            }

            Ok(Value::Bool(false))
        }
        _ => Err(ASTError::ValueTypeMismatch),
    }
}

fn only_include_charset(
    key: String,
    arguments: &[Expression],
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    assert_param_expression!(format!("{}.arguments[0]", key), arguments[0], Expression::Variable(VariableExpression {
        name: VarName::AccountChars
    }), format!("variable AccountChars"));

    let charset = handle_expression(key.clone() + ".arguments[1]", &arguments[1], account_chars, account)?;

    assert_param_expression!(format!("{}.arguments[1]", key), charset, Value::CharsetType(_), format!("charset_type"));

    let expected_charset = match charset {
        Value::CharsetType(charset) => charset,
        _ => return Err(ASTError::ValueTypeMismatch),
    };

    for item in account_chars.iter() {
        let charset_index = u32::from(item.char_set_name());
        let charset = CharSetType::try_from(charset_index).map_err(|_| ASTError::UndefinedCharSetType {
            key: "".to_string(),
            type_: charset_index,
        })?;

        if expected_charset != charset {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}

fn in_list(
    key: String,
    arguments: &[Expression],
    account_chars: packed::AccountCharsReader,
    account: &str,
) -> Result<Value, ASTError> {
    assert_param_expression!(format!("{}.arguments[0]", key), arguments[0], Expression::Variable(VariableExpression {
        name: VarName::Account
    }), format!("variable AccountChars"));

    let account_list = handle_expression(key.clone() + ".arguments[1]", &arguments[1], account_chars, account)?;

    assert_param_expression!(format!("{}.arguments[1]", key), account_list, Value::BinaryVec(_), format!("binary[]"));

    match account_list {
        Value::BinaryVec(account_list) => {
            let hash = blake2b_256(account);
            let account_id = hash[0..20].to_vec();
            // println!("account_id = {:?}", hex::encode(&account_id));
            Ok(Value::Bool(account_list.contains(&account_id)))
        }
        _ => Err(ASTError::ValueTypeMismatch),
    }
}

#[cfg(test)]
mod test {
    use das_types_std::types;
    use serde_json::json;

    use super::*;
    use crate::util;

    #[test]
    fn playground() {
        let rules_json = json!([
            {
                "index": 0,
                "name": "Price of 1 Charactor Emoji DID",
                "note": "",
                "price": 100_000_000,
                "status": 1,
                "ast": {
                    "type": "operator",
                    "symbol": "and",
                    "expressions": [
                        {
                            "type": "operator",
                            "symbol": "==",
                            "expressions": [
                                {
                                    "type": "variable",
                                    "name": "account_length",
                                },
                                {
                                    "type": "value",
                                    "value_type": "uint32",
                                    "value": 1,
                                },
                            ],
                        },
                        {
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
                                }
                            ],
                        }
                    ]
                }
            }
        ]);

        let rules = util::json_to_sub_account_rules(String::new(), &rules_json).unwrap();

        let mut dummy_account_chars_builder = packed::AccountChars::new_builder();
        dummy_account_chars_builder = dummy_account_chars_builder.push(packed::AccountChar::default());
        let dummy_account_chars = dummy_account_chars_builder.build();
        let dummy_account = "";

        let ret = match_rule_with_account_chars(&rules, dummy_account_chars.as_reader(), dummy_account);
        println!("return: {:?}", ret);
        if let Err(err) = ret.as_ref() {
            println!("error msg: {:?}\n", err.to_string());
        }

        assert!(ret.is_ok());
    }

    #[test]
    fn test_ast_not_function_or_operator() {
        let rules = vec![SubAccountRule {
            index: 0,
            name: "".to_string(),
            note: "".to_string(),
            price: 0,
            status: SubAccountRuleStatus::On,
            ast: Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            }),
        }];

        let ret = match_rule_with_account_chars(&rules, packed::AccountChars::default().as_reader(), "");
        assert!(ret.is_err());
        assert!(matches!(ret.unwrap_err(), ASTError::FunctionOrOperatorRequired { .. }));
    }

    #[test]
    fn test_disabled_rule_skipping() {
        let rules = vec![
            SubAccountRule {
                index: 0,
                name: "".to_string(),
                note: "".to_string(),
                price: 0,
                status: SubAccountRuleStatus::Off,
                ast: Expression::Operator(OperatorExpression {
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
                }),
            },
            SubAccountRule {
                index: 1,
                name: "".to_string(),
                note: "".to_string(),
                price: 0,
                status: SubAccountRuleStatus::On,
                ast: Expression::Operator(OperatorExpression {
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
                }),
            },
        ];

        let ret = match_rule_with_account_chars(&rules, packed::AccountChars::default().as_reader(), "").unwrap();
        assert!(ret.is_some());

        // rules[0] is disabled, so the matched rule should be rules[1]
        let rule = ret.unwrap();
        assert_eq!(1, rule.index);
    }

    fn test_operator_expression(expression: Expression) -> Value {
        let key = String::from(".");
        let account_chars = packed::AccountChars::default();
        let account = "";

        handle_expression(key, &expression, account_chars.as_reader(), account).unwrap()
    }

    fn test_err_operator_expression(expression: Expression) -> Result<Value, ASTError> {
        let key = String::from(".");
        let account_chars = packed::AccountChars::default();
        let account = "";

        handle_expression(key, &expression, account_chars.as_reader(), account)
    }

    #[test]
    fn test_operator_and() {
        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
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
        }));
        assert!(matches!(ret, Value::Bool(true)));

        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
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
        }));
        assert!(matches!(ret, Value::Bool(false)));

        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                }),
            ],
        }));
        assert!(matches!(ret, Value::Bool(false)));
    }

    #[test]
    fn test_operator_and_param_type_error() {
        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Uint32,
                    value: Value::Uint32(1),
                }),
            ],
        }));
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::And,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
            ],
        }));
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }

    #[test]
    fn test_operator_or() {
        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Or,
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
        }));
        assert!(matches!(ret, Value::Bool(true)));

        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Or,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(false),
                }),
            ],
        }));
        assert!(matches!(ret, Value::Bool(false)));

        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Or,
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
        }));
        assert!(matches!(ret, Value::Bool(true)));
    }

    #[test]
    fn test_operator_or_param_error() {
        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Or,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Uint32,
                    value: Value::Uint32(1),
                }),
            ],
        }));
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Or,
            expressions: vec![
                Expression::Value(ValueExpression {
                    value_type: ValueType::Bool,
                    value: Value::Bool(true),
                }),
            ],
        }));
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }

    #[test]
    fn test_operator_not() {
        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Not,
            expressions: vec![Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            })],
        }));
        assert!(matches!(ret, Value::Bool(false)));

        let ret = test_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Not,
            expressions: vec![Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(false),
            })],
        }));
        assert!(matches!(ret, Value::Bool(true)));
    }

    #[test]
    fn test_operator_not_param_error() {
        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Not,
            expressions: vec![Expression::Value(ValueExpression {
                value_type: ValueType::Uint32,
                value: Value::Uint32(1),
            })],
        }));
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
            symbol: SymbolType::Not,
            expressions: vec![Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(false),
            }), Expression::Value(ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(false),
            })],
        }));
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }

    macro_rules! gen_compare_test {
        (all $symbol_type: expr, $value_type: expr, $value_ty: ident => $ret_1: expr, $ret_2: expr, $ret_3: expr) => {
            gen_compare_test!(
                single
                $symbol_type,
                $value_type,
                Value::$value_ty(1),
                Value::$value_ty(0),
                $ret_1
            );
            gen_compare_test!(
                single
                $symbol_type,
                $value_type,
                Value::$value_ty(0),
                Value::$value_ty(1),
                $ret_2
            );
            gen_compare_test!(
                single
                $symbol_type,
                $value_type,
                Value::$value_ty(1),
                Value::$value_ty(1),
                $ret_3
            );
        };
        (single $symbol_type: expr, $value_type: expr, $value_1: expr, $value_2: expr, $result: expr) => {
            let ret = test_operator_expression(Expression::Operator(OperatorExpression {
                symbol: $symbol_type,
                expressions: vec![
                    Expression::Value(ValueExpression {
                        value_type: $value_type,
                        value: $value_1,
                    }),
                    Expression::Value(ValueExpression {
                        value_type: $value_type,
                        value: $value_2,
                    }),
                ],
            }));
            assert!(matches!(ret, Value::Bool($result)));
        };
        (all_err $symbol_type: expr) => {
            gen_compare_test!(type_err $symbol_type, ValueExpression {
                value_type: ValueType::Bool,
                value: Value::Bool(true),
            }, ValueExpression {
                value_type: ValueType::Uint32,
                value: Value::Uint32(1),
            } => ASTError::ParamTypeError { key: _, types: _ });

            gen_compare_test!(length_err $symbol_type, ValueExpression {
                value_type: ValueType::Uint8,
                value: Value::Uint8(1),
            } => ASTError::ParamLengthError { key: _, expected_length: _, length: _ });
        };
        (type_err $symbol_type: expr, $value_1: expr, $value_2: expr => $error: pat_param) => {
            let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
                symbol: $symbol_type,
                expressions: vec![Expression::Value($value_1), Expression::Value($value_2)],
            }));
            // println!("ret = {:?}", ret);
            assert!(matches!(ret, Err($error)));
        };
        (length_err $symbol_type: expr, $value_1: expr => $error: pat_param) => {
            let ret = test_err_operator_expression(Expression::Operator(OperatorExpression {
                symbol: $symbol_type,
                expressions: vec![
                    Expression::Value($value_1),
                    Expression::Value($value_1),
                    Expression::Value($value_1),
                ],
            }));
            // println!("ret = {:?}", ret);
            assert!(matches!(ret, Err($error)));
        };
    }

    #[test]
    fn test_operator_gt() {
        gen_compare_test!(all SymbolType::Gt, ValueType::Uint8, Uint8 => true, false, false);
        gen_compare_test!(all SymbolType::Gt, ValueType::Uint32, Uint32 => true, false, false);
        gen_compare_test!(all SymbolType::Gt, ValueType::Uint64, Uint64 => true, false, false);
    }

    #[test]
    fn test_operator_gt_param_error() {
        gen_compare_test!(all_err SymbolType::Gt);
    }

    #[test]
    fn test_operator_gte() {
        gen_compare_test!(all SymbolType::Gte, ValueType::Uint8, Uint8 => true, false, true);
        gen_compare_test!(all SymbolType::Gte, ValueType::Uint32, Uint32 => true, false, true);
        gen_compare_test!(all SymbolType::Gte, ValueType::Uint64, Uint64 => true, false, true);
    }

    #[test]
    fn test_operator_gte_param_error() {
        gen_compare_test!(all_err SymbolType::Gte);
    }

    #[test]
    fn test_operator_lt() {
        gen_compare_test!(all SymbolType::Lt, ValueType::Uint8, Uint8 => false, true, false);
        gen_compare_test!(all SymbolType::Lt, ValueType::Uint32, Uint32 => false, true, false);
        gen_compare_test!(all SymbolType::Lt, ValueType::Uint64, Uint64 => false, true, false);
    }

    #[test]
    fn test_operator_lt_param_error() {
        gen_compare_test!(all_err SymbolType::Lt);
    }

    #[test]
    fn test_operator_lte() {
        gen_compare_test!(all SymbolType::Lte, ValueType::Uint8, Uint8 => false, true, true);
        gen_compare_test!(all SymbolType::Lte, ValueType::Uint32, Uint32 => false, true, true);
        gen_compare_test!(all SymbolType::Lte, ValueType::Uint64, Uint64 => false, true, true);
    }

    #[test]
    fn test_operator_lte_param_error() {
        gen_compare_test!(all_err SymbolType::Lte);
    }

    #[test]
    fn test_operator_eq() {
        gen_compare_test!(all SymbolType::Equal, ValueType::Uint8, Uint8 => false, false, true);
        gen_compare_test!(all SymbolType::Equal, ValueType::Uint32, Uint32 => false, false, true);
        gen_compare_test!(all SymbolType::Equal, ValueType::Uint64, Uint64 => false, false, true);
    }

    #[test]
    fn test_operator_eq_param_error() {
        gen_compare_test!(all_err SymbolType::Equal);
    }

    fn test_function_expression(expression: Expression, account_chars: types::AccountChars, account: &str) -> Value {
        let key = String::from(".");
        let account_chars: packed::AccountChars = account_chars.into();

        handle_expression(key, &expression, account_chars.as_reader(), account).unwrap()
    }

    fn test_err_function_expression(expression: Expression, account_chars: types::AccountChars, account: &str) -> Result<Value, ASTError> {
        let key = String::from(".");
        let account_chars: packed::AccountChars = account_chars.into();

        handle_expression(key, &expression, account_chars.as_reader(), account)
    }

    #[test]
    fn test_function_include_chars() {
        fn inner(string_vec: Vec<String>, account: &str) -> Value {
            test_function_expression(Expression::Function(FunctionExpression {
                name: FnName::IncludeChars,
                arguments: vec![
                    Expression::Variable(VariableExpression { name: VarName::Account }),
                    Expression::Value(ValueExpression {
                        value_type: ValueType::StringVec,
                        value: Value::StringVec(string_vec),
                    }),
                ]
            }), packed::AccountChars::default().into(), account)
        }

        let ret = inner(vec!["🌈".to_string(), "✨".to_string()], "xxxxx.ast.bit");
        assert!(matches!(ret, Value::Bool(false)));

        let ret = inner(vec!["🌈".to_string(), "✨".to_string()], "xxxx🌈.ast.bit");
        assert!(matches!(ret, Value::Bool(true)));

        let ret = inner(vec!["uni".to_string(), "meta".to_string()], "xxxxxxx.ast.bit");
        assert!(matches!(ret, Value::Bool(false)));

        let ret = inner(vec!["uni".to_string(), "meta".to_string()], "metaverse.ast.bit");
        assert!(matches!(ret, Value::Bool(true)));
    }

    #[test]
    fn test_function_include_chars_param_error() {
        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::IncludeChars,
            arguments: vec![
                Expression::Variable(VariableExpression { name: VarName::AccountChars }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::StringVec,
                    value: Value::StringVec(vec!["uni".to_string(), "meta".to_string()]),
                }),
            ]
        }), vec![], "xxxxxxx.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::IncludeChars,
            arguments: vec![
                Expression::Variable(VariableExpression { name: VarName::Account }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::String,
                    value: Value::String("uni".to_string()),
                }),
            ]
        }), vec![], "xxxxxxx.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::IncludeChars,
            arguments: vec![
                Expression::Variable(VariableExpression { name: VarName::Account }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::StringVec,
                    value: Value::StringVec(vec!["uni".to_string(), "meta".to_string()]),
                }),
                Expression::Variable(VariableExpression { name: VarName::Account }),
            ]
        }), vec![], "xxxxxxx.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }

    #[test]
    fn test_function_only_include_charset() {
        fn inner(account_chars: types::AccountChars, account: &str) -> Value {
            test_function_expression(Expression::Function(FunctionExpression {
                name: FnName::OnlyIncludeCharset,
                arguments: vec![
                    Expression::Variable(VariableExpression {
                        name: VarName::AccountChars,
                    }),
                    Expression::Value(ValueExpression {
                        value_type: ValueType::CharsetType,
                        value: Value::CharsetType(CharSetType::Digit),
                    }),
                ]
            }), account_chars, account)
        }

        let ret = inner(vec![
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Emoji,
                char: String::new(),
            },
        ], "111✨.ast.bit");
        assert!(matches!(ret, Value::Bool(false)));

        let ret = inner(vec![
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
        ], "1111.ast.bit");
        assert!(matches!(ret, Value::Bool(true)));
    }

    #[test]
    fn test_function_only_include_charset_param_error() {
        let account_chars = vec![
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
            types::AccountChar {
                char_set_type: CharSetType::Digit,
                char: String::new(),
            },
        ];

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::Account,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::CharsetType,
                    value: Value::CharsetType(CharSetType::Digit),
                }),
            ]
        }), account_chars.clone(), "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::String,
                    value: Value::String("test".to_string()),
                }),
            ]
        }), account_chars.clone(), "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::OnlyIncludeCharset,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::CharsetType,
                    value: Value::CharsetType(CharSetType::Digit),
                }),
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
            ]
        }), account_chars.clone(), "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }

    #[test]
    fn test_function_in_list() {
        fn inner(list: Vec<Binary>, account: &str) -> Value {
            test_function_expression(Expression::Function(FunctionExpression {
                name: FnName::InList,
                arguments: vec![
                    Expression::Variable(VariableExpression {
                        name: VarName::Account,
                    }),
                    Expression::Value(ValueExpression {
                        value_type: ValueType::BinaryVec,
                        value: Value::BinaryVec(list),
                    }),
                ]
            }), vec![], account)
        }

        let ret = inner(
            vec![
                hex::decode("0000000000000000000000000000000000000000").unwrap(),
                hex::decode("0000000000000000000000000000000000000001").unwrap(),
                hex::decode("0000000000000000000000000000000000000002").unwrap(),
                hex::decode("80165a04a62a5328e0b95ed3301ee4837e8075f7").unwrap(),
            ],
            "1111.ast.bit",
        );
        assert!(matches!(ret, Value::Bool(true)));

        let ret = inner(
            vec![
                hex::decode("0000000000000000000000000000000000000000").unwrap(),
                hex::decode("0000000000000000000000000000000000000001").unwrap(),
                hex::decode("0000000000000000000000000000000000000002").unwrap(),
            ],
            "1111.ast.bit",
        );
        assert!(matches!(ret, Value::Bool(false)));
    }

    #[test]
    fn test_function_in_list_param_error() {
        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::InList,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::AccountChars,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::BinaryVec,
                    value: Value::BinaryVec(vec![]),
                }),
            ]
        }), vec![], "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::InList,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::Account,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::Binary,
                    value: Value::Binary(vec![]),
                }),
            ]
        }), vec![], "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamTypeError { key: _, types: _ })));

        let ret = test_err_function_expression(Expression::Function(FunctionExpression {
            name: FnName::InList,
            arguments: vec![
                Expression::Variable(VariableExpression {
                    name: VarName::Account,
                }),
                Expression::Value(ValueExpression {
                    value_type: ValueType::BinaryVec,
                    value: Value::BinaryVec(vec![]),
                }),
                Expression::Variable(VariableExpression {
                    name: VarName::Account,
                }),
            ]
        }), vec![], "1111.ast.bit");
        assert!(matches!(ret, Err(ASTError::ParamLengthError { key: _, expected_length: _, length: _ })));
    }
}
